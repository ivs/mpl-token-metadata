use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_pack::IsInitialized, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};
use spl_token_2022::{
    extension::{
        metadata_pointer::{self, MetadataPointer},
        mint_close_authority::MintCloseAuthority,
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction::initialize_mint_close_authority,
    native_mint::DECIMALS,
    state::Mint,
};

use crate::{
    error::MetadataError,
    instruction::{Context, Create, CreateArgs},
    state::{
        Metadata, ProgrammableConfig, TokenMetadataAccount, TokenStandard, MAX_MASTER_EDITION_LEN,
        TOKEN_STANDARD_INDEX,
    },
    utils::{
        create_master_edition,
        fee::{levy, set_fee_flag, LevyArgs},
        process_create_metadata_accounts_logic, CreateMetadataAccountsLogicArgs,
    },
};

/// List of SPL Token-2022 `Mint` account exntesion types that are allowed on
/// non-fungible assets.
const NON_FUNGIBLE_EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::MintCloseAuthority,
    ExtensionType::NonTransferable,
    ExtensionType::MetadataPointer,
];

/// Create the associated metadata accounts for a mint.
///
/// The instruction will also initialize the mint if the account does not
/// exist. For `NonFungible` assets, a `master_edition` account is required.
pub fn create<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: CreateArgs,
) -> ProgramResult {
    let context = Create::to_context(accounts)?;

    match args {
        CreateArgs::V1 { .. } => create_v1(program_id, context, args),
    }
}

/// V1 implementation of the create instruction.
fn create_v1(program_id: &Pubkey, ctx: Context<Create>, args: CreateArgs) -> ProgramResult {
    // get the args for the instruction
    let CreateArgs::V1 {
        ref asset_data,
        decimals,
        print_supply,
    } = args;

    // cannot create non-fungible editions on this instruction
    if matches!(
        asset_data.token_standard,
        TokenStandard::NonFungibleEdition | TokenStandard::ProgrammableNonFungibleEdition
    ) {
        return Err(MetadataError::InvalidTokenStandard.into());
    }

    // Levy fees first, to fund the metadata account with rent + fee amount.
    levy(LevyArgs {
        payer_account_info: ctx.accounts.payer_info,
        token_metadata_pda_info: ctx.accounts.metadata_info,
    })?;

    // if the account does not exist, we will allocate a new mint
    if ctx.accounts.mint_info.data_is_empty() {
        // mint account must be a signer in the transaction
        if !ctx.accounts.mint_info.is_signer {
            return Err(MetadataError::MintIsNotSigner.into());
        }

        let spl_token_program = ctx
            .accounts
            .spl_token_program_info
            .ok_or(MetadataError::MissingSplTokenProgram)?;

        create_mint(
            ctx.accounts.mint_info,
            ctx.accounts.metadata_info,
            ctx.accounts.authority_info,
            ctx.accounts.payer_info,
            asset_data.token_standard,
            decimals,
            spl_token_program,
        )?;
    } else {
        validate_mint(
            ctx.accounts.mint_info,
            ctx.accounts.metadata_info,
            asset_data.token_standard,
        )?;
    }

    // creates the metadata account

    process_create_metadata_accounts_logic(
        program_id,
        CreateMetadataAccountsLogicArgs {
            metadata_account_info: ctx.accounts.metadata_info,
            mint_info: ctx.accounts.mint_info,
            mint_authority_info: ctx.accounts.authority_info,
            payer_account_info: ctx.accounts.payer_info,
            update_authority_info: ctx.accounts.update_authority_info,
            system_account_info: ctx.accounts.system_program_info,
        },
        asset_data.as_data_v2(),
        false,
        asset_data.is_mutable,
        false,
        true,
        asset_data.collection_details.clone(),
        None,
    )?;

    // creates the master edition account (only for NonFungible assets)

    if matches!(
        asset_data.token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        let print_supply = print_supply.ok_or(MetadataError::MissingPrintSupply)?;

        if let Some(master_edition) = ctx.accounts.master_edition_info {
            let spl_token_program = ctx
                .accounts
                .spl_token_program_info
                .ok_or(MetadataError::MissingSplTokenProgram)?;

            create_master_edition(
                program_id,
                master_edition,
                ctx.accounts.mint_info,
                ctx.accounts.update_authority_info,
                ctx.accounts.authority_info,
                ctx.accounts.payer_info,
                ctx.accounts.metadata_info,
                spl_token_program,
                ctx.accounts.system_program_info,
                print_supply.to_option(),
            )?;

            // for pNFTs, we store the token standard value at the end of the
            // master edition account
            if matches!(
                asset_data.token_standard,
                TokenStandard::ProgrammableNonFungible
            ) {
                let mut data = master_edition.data.borrow_mut();

                if data.len() < MAX_MASTER_EDITION_LEN {
                    return Err(MetadataError::InvalidMasterEditionAccountLength.into());
                }

                data[TOKEN_STANDARD_INDEX] = TokenStandard::ProgrammableNonFungible as u8;
            }
        } else {
            return Err(MetadataError::MissingMasterEditionAccount.into());
        }
    } else if print_supply.is_some() {
        msg!("Ignoring print supply for selected token standard");
    }

    let mut metadata = Metadata::from_account_info(ctx.accounts.metadata_info)?;
    metadata.token_standard = Some(asset_data.token_standard);
    metadata.primary_sale_happened = asset_data.primary_sale_happened;

    // sets the programmable config for programmable assets

    if matches!(
        asset_data.token_standard,
        TokenStandard::ProgrammableNonFungible
    ) {
        metadata.programmable_config = Some(ProgrammableConfig::V1 {
            rule_set: asset_data.rule_set,
        });
    }

    // saves the metadata state
    metadata.save(&mut ctx.accounts.metadata_info.try_borrow_mut_data()?)?;

    // Set fee flag after metadata account is created.
    set_fee_flag(ctx.accounts.metadata_info)
}

/// Validates the mint account for the given token standard.
///
/// For all token standards, the validation consists of checking that the mint:
/// - is initialized
/// - (token-2022) has the mint close authority extension enabled and set to the metadata account
///
/// For non-fungibles assets, the validation consists of checking that the mint:
/// - has no more than 1 supply
/// - has 0 decimals
/// - (token-2022) has no other extensions enabled apart from `ExtensionType::MintCloseAuthority`
///    and `ExtensionType::NonTransferable`
///
/// For programmable non-fungibles assets, the validation consists of checking that the mint:
/// - has no more than 0 supply
/// - has 0 decimals
/// - (token-2022) has no other extensions enabled apart from `MintCloseAuthority`
///    and `NonTransferable`
fn validate_mint(
    mint: &AccountInfo,
    metadata: &AccountInfo,
    token_standard: TokenStandard,
) -> ProgramResult {
    let mint_data = &mint.data.borrow();
    let mint = StateWithExtensions::<Mint>::unpack(mint_data)?;

    if !mint.base.is_initialized() {
        return Err(MetadataError::Uninitialized.into());
    }

    if matches!(
        token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        // NonFungible assets must have decimals == 0 and supply no greater than 1
        if mint.base.decimals > 0 || mint.base.supply > 1 {
            return Err(MetadataError::InvalidMintForTokenStandard.into());
        }
        // Programmable assets must have supply == 0 since there cannot be any
        // existing token account
        if matches!(token_standard, TokenStandard::ProgrammableNonFungible)
            && (mint.base.supply > 0)
        {
            return Err(MetadataError::MintSupplyMustBeZero.into());
        }

        // validates the mint extensions
        mint.get_extension_types()?
            .iter()
            .try_for_each(|extension_type| {
                if !NON_FUNGIBLE_EXTENSIONS.contains(extension_type) {
                    msg!("Invalid mint extension: {:?}", extension_type);
                    return Err(MetadataError::InvalidMintExtensionType);
                }
                Ok(())
            })?;
    }

    // For all token standards:
    //
    // 1) if the mint close authority extension is enabled, it must
    //    be set to be the metadata account; and
    if let Ok(extension) = mint.get_extension::<MintCloseAuthority>() {
        let close_authority: Option<Pubkey> = extension.close_authority.into();
        if close_authority.is_none() || close_authority != Some(*metadata.key) {
            return Err(MetadataError::InvalidMintCloseAuthority.into());
        }
    }

    // 2) if the metadata pointer extension is enabled, it must be set
    //    to the metadata account address
    if let Ok(extension) = mint.get_extension::<MetadataPointer>() {
        let authority: Option<Pubkey> = extension.authority.into();
        let metadata_address: Option<Pubkey> = extension.metadata_address.into();

        if authority.is_some() {
            msg!("Metadata pointer extension: authority must be None");
            return Err(MetadataError::InvalidMetadataPointer.into());
        }

        if metadata_address != Some(*metadata.key) {
            msg!("Metadata pointer extension: metadata address mismatch");
            return Err(MetadataError::InvalidMetadataPointer.into());
        }
    }

    Ok(())
}

fn create_mint<'a>(
    mint: &'a AccountInfo<'a>,
    metadata: &'a AccountInfo<'a>,
    authority: &'a AccountInfo<'a>,
    payer: &'a AccountInfo<'a>,
    token_standard: TokenStandard,
    decimals: Option<u8>,
    spl_token_program: &'a AccountInfo<'a>,
) -> ProgramResult {
    let spl_token_2022 = matches!(spl_token_program.key, &spl_token_2022::ID);

    let mint_account_size = if spl_token_2022 {
        ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::MintCloseAuthority,
            ExtensionType::MetadataPointer,
        ])?
    } else {
        Mint::LEN
    };

    invoke(
        &system_instruction::create_account(
            payer.key,
            mint.key,
            Rent::get()?.minimum_balance(mint_account_size),
            mint_account_size as u64,
            spl_token_program.key,
        ),
        &[payer.clone(), mint.clone()],
    )?;

    if spl_token_2022 {
        let account_infos = vec![mint.clone(), metadata.clone(), spl_token_program.clone()];

        invoke(
            &initialize_mint_close_authority(spl_token_program.key, mint.key, Some(metadata.key))?,
            &account_infos,
        )?;

        invoke(
            &metadata_pointer::instruction::initialize(
                spl_token_program.key,
                mint.key,
                None,
                Some(*metadata.key),
            )?,
            &account_infos,
        )?;
    }

    let decimals = match token_standard {
        // for NonFungible variants, we ignore the argument and
        // always use 0 decimals
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible => 0,
        // for Fungile variants, we either use the specified decimals or the default
        // DECIMALS from spl-token
        TokenStandard::FungibleAsset | TokenStandard::Fungible => match decimals {
            Some(decimals) => decimals,
            // if decimals not provided, use the default
            None => DECIMALS,
        },
        _ => {
            return Err(MetadataError::InvalidTokenStandard.into());
        }
    };

    // initializing the mint account
    invoke(
        &spl_token_2022::instruction::initialize_mint2(
            spl_token_program.key,
            mint.key,
            authority.key,
            Some(authority.key),
            decimals,
        )?,
        &[mint.clone(), authority.clone()],
    )
}
