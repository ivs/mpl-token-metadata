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

// Accounts.
export type CollectInstructionAccounts = {
  /** Authority to collect fees */
  authority?: Signer;
  /** PDA to retrieve fees from */
  pdaAccount: PublicKey | Pda;
};

// Data.
export type CollectInstructionData = { discriminator: number };

export type CollectInstructionDataArgs = {};

export function getCollectInstructionDataSerializer(): Serializer<
  CollectInstructionDataArgs,
  CollectInstructionData
> {
  return mapSerializer<CollectInstructionDataArgs, any, CollectInstructionData>(
    struct<CollectInstructionData>([['discriminator', u8()]], {
      description: 'CollectInstructionData',
    }),
    (value) => ({ ...value, discriminator: 54 })
  ) as Serializer<CollectInstructionDataArgs, CollectInstructionData>;
}

// Instruction.
export function collect(
  context: Pick<Context, 'identity' | 'programs'>,
  input: CollectInstructionAccounts
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplTokenMetadata',
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
  );

  // Accounts.
  const resolvedAccounts = {
    authority: {
      index: 0,
      isWritable: false as boolean,
      value: input.authority ?? null,
    },
    pdaAccount: {
      index: 1,
      isWritable: false as boolean,
      value: input.pdaAccount ?? null,
    },
  } satisfies ResolvedAccountsWithIndices;

  // Default values.
  if (!resolvedAccounts.authority.value) {
    resolvedAccounts.authority.value = context.identity;
  }

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'programId',
    programId
  );

  // Data.
  const data = getCollectInstructionDataSerializer().serialize({});

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
