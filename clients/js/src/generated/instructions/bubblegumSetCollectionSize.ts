/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  mapSerializer,
  struct,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';
import {
  SetCollectionSizeArgs,
  SetCollectionSizeArgsArgs,
  getSetCollectionSizeArgsSerializer,
} from '../types';

// Accounts.
export type BubblegumSetCollectionSizeInstructionAccounts = {
  /** Collection Metadata account */
  collectionMetadata: PublicKey | Pda;
  /** Collection Update authority */
  collectionAuthority: Signer;
  /** Mint of the Collection */
  collectionMint: PublicKey | Pda;
  /** Signing PDA of Bubblegum program */
  bubblegumSigner: Signer;
  /** Collection Authority Record PDA */
  collectionAuthorityRecord?: PublicKey | Pda;
};

// Data.
export type BubblegumSetCollectionSizeInstructionData = {
  discriminator: number;
  setCollectionSizeArgs: SetCollectionSizeArgs;
};

export type BubblegumSetCollectionSizeInstructionDataArgs = {
  setCollectionSizeArgs: SetCollectionSizeArgsArgs;
};

export function getBubblegumSetCollectionSizeInstructionDataSerializer(): Serializer<
  BubblegumSetCollectionSizeInstructionDataArgs,
  BubblegumSetCollectionSizeInstructionData
> {
  return mapSerializer<
    BubblegumSetCollectionSizeInstructionDataArgs,
    any,
    BubblegumSetCollectionSizeInstructionData
  >(
    struct<BubblegumSetCollectionSizeInstructionData>(
      [
        ['discriminator', u8()],
        ['setCollectionSizeArgs', getSetCollectionSizeArgsSerializer()],
      ],
      { description: 'BubblegumSetCollectionSizeInstructionData' }
    ),
    (value) => ({ ...value, discriminator: 36 })
  ) as Serializer<
    BubblegumSetCollectionSizeInstructionDataArgs,
    BubblegumSetCollectionSizeInstructionData
  >;
}

// Args.
export type BubblegumSetCollectionSizeInstructionArgs =
  BubblegumSetCollectionSizeInstructionDataArgs;

// Instruction.
export function bubblegumSetCollectionSize(
  context: Pick<Context, 'programs'>,
  input: BubblegumSetCollectionSizeInstructionAccounts &
    BubblegumSetCollectionSizeInstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplTokenMetadata',
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
  );

  // Accounts.
  const resolvedAccounts = {
    collectionMetadata: {
      index: 0,
      isWritable: true as boolean,
      value: input.collectionMetadata ?? null,
    },
    collectionAuthority: {
      index: 1,
      isWritable: true as boolean,
      value: input.collectionAuthority ?? null,
    },
    collectionMint: {
      index: 2,
      isWritable: false as boolean,
      value: input.collectionMint ?? null,
    },
    bubblegumSigner: {
      index: 3,
      isWritable: false as boolean,
      value: input.bubblegumSigner ?? null,
    },
    collectionAuthorityRecord: {
      index: 4,
      isWritable: false as boolean,
      value: input.collectionAuthorityRecord ?? null,
    },
  } satisfies ResolvedAccountsWithIndices;

  // Arguments.
  const resolvedArgs: BubblegumSetCollectionSizeInstructionArgs = { ...input };

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'omitted',
    programId
  );

  // Data.
  const data =
    getBubblegumSetCollectionSizeInstructionDataSerializer().serialize(
      resolvedArgs as BubblegumSetCollectionSizeInstructionDataArgs
    );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
