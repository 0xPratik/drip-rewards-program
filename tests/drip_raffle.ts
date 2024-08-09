import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DripRaffle } from "../target/types/drip_raffle";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  getAssociatedTokenAddress,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  createCreateMasterEditionV3Instruction,
  createCreateMetadataAccountV3Instruction,
  CreateMetadataAccountArgsV3,
  DataV2,
} from "@metaplex-foundation/mpl-token-metadata";
import { PROGRAM_ID as BUBBLEGUM_PROGRAM_ID } from "@metaplex-foundation/mpl-bubblegum";
import {
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  SPL_NOOP_PROGRAM_ID,
} from "@solana/spl-account-compression";
import { createAndMint } from "./createAndMint";
import { getcNFTsFromCollection } from "./fetchNFTsByCollection";
import { getAsset, getAssetProof } from "./readApi";
import { decode, mapProof } from "./utils";

export const RPC_PATH = "https://api.devnet.solana.com";

describe("drip_raffle", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DripRaffle as Program<DripRaffle>;
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet as anchor.Wallet;

  const mintKey = new anchor.web3.Keypair();

  const end_date_sec = Math.floor(Date.now() / 1000);
  const final_end_date = end_date_sec + 86400;
  const end_date = final_end_date;
  const max_entries = 100;
  const collectionItem = {
    mintAddress: new anchor.web3.PublicKey(
      "ELxUQkWLMBCoMatry9qzQR6RHiUYmVndUpmPwhZ8PKJK"
    ),
    weight: 1,
  };

  let treeAddress: anchor.web3.PublicKey | undefined = undefined;
  let treeAuthority: anchor.web3.PublicKey | undefined = undefined;
  let assetId: string | undefined = undefined;

  let [raffle_account, raffle_bump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("raffle"),
        wallet.publicKey.toBuffer(),
        Buffer.from(new BigUint64Array([BigInt(end_date)]).buffer),
      ],
      program.programId
    );
  const collection = [collectionItem];

  let cnftCollection: anchor.web3.PublicKey | undefined = undefined;

  it("create raffle", async () => {
    // Add your test here.

    const tx = await program.methods
      .createRaffle(new anchor.BN(end_date), collection)
      .accounts({
        authority: wallet.publicKey,
        raffleAccount: raffle_account,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({
        skipPreflight: true,
      });

    console.log("Create Raffle Tx", tx);
  });
  it("should mint a nft", async () => {
    const { blockhash } = await program.provider.connection.getLatestBlockhash(
      "finalized"
    );
    const reciever = wallet.publicKey;
    const transaction = new anchor.web3.Transaction({
      recentBlockhash: blockhash,
      feePayer: wallet.publicKey,
    });
    const lamports =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        MINT_SIZE
      );
    transaction.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: wallet.publicKey, // The account that will transfer lamports to the created account
        newAccountPubkey: mintKey.publicKey, // Public key of the created account
        space: MINT_SIZE, // Amount of space in bytes to allocate to the created account
        lamports, // Amount of lamports to transfer to the created account
        programId: TOKEN_PROGRAM_ID, // Public key of the program to assign as the owner of the created account
      }),
      createInitializeMintInstruction(
        mintKey.publicKey, // mint pubkey
        0, // decimals
        wallet.publicKey, // mint authority
        wallet.publicKey // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      )
    );

    let wallet_ata = await getAssociatedTokenAddress(
      mintKey.publicKey, // mint
      reciever // owner
    );

    transaction.add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        wallet_ata,
        reciever,
        mintKey.publicKey
      ),
      createMintToInstruction(
        mintKey.publicKey, // mint
        wallet_ata,
        wallet.publicKey,
        1
      )
    );

    const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
      "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
    );

    const [metadatakey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKey.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const [masterKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKey.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const data: DataV2 = {
      name: "Metaplex 2",
      symbol: "PAT",
      uri: "https://metadata.degods.com/g/2342.json",
      sellerFeeBasisPoints: 500,
      creators: [
        {
          address: wallet.publicKey,
          verified: true,
          share: 100,
        },
      ],
      collection: null,
      uses: null,
    };

    const args = {
      data,
      isMutable: true,
    };

    const argsV3: CreateMetadataAccountArgsV3 = {
      data,
      isMutable: true,
      collectionDetails: null,
    };

    const createMetadataV3 = createCreateMetadataAccountV3Instruction(
      {
        metadata: metadatakey,
        mint: mintKey.publicKey,
        mintAuthority: wallet.publicKey,
        payer: wallet.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        updateAuthority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      {
        createMetadataAccountArgsV3: argsV3,
      }
    );

    transaction.add(createMetadataV3);

    const createMasterEditionV3 = createCreateMasterEditionV3Instruction(
      {
        edition: masterKey,
        mint: mintKey.publicKey,
        updateAuthority: wallet.publicKey,
        mintAuthority: wallet.publicKey,
        payer: wallet.publicKey,
        metadata: metadatakey,
      },
      {
        createMasterEditionArgs: {
          maxSupply: new anchor.BN(0),
        },
      }
    );
    transaction.add(createMasterEditionV3);

    transaction.partialSign(mintKey);
    const signed_transaction = await wallet.signTransaction(transaction);
    const serialized_transaction = signed_transaction.serialize();

    const sig = await program.provider.connection.sendRawTransaction(
      serialized_transaction
    );
    await program.provider.connection.confirmTransaction(sig, "confirmed");
    console.log("Transaction Signature MINTED NFT", sig);
  });
  it("shold add reward", async () => {
    let reward_owner_account = await getAssociatedTokenAddress(
      mintKey.publicKey, // mint
      wallet.publicKey, // owner
      true
    );

    let raffle_treasury_account = await getAssociatedTokenAddress(
      mintKey.publicKey, // mint
      raffle_account, // owner
      true
    );

    const tx = await program.methods
      .depositNft()
      .accounts({
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        owner: wallet.publicKey,
        rewardMint: mintKey.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        raffleAccount: raffle_account,
        rewardOwnerAccount: reward_owner_account,
        rewardRaffleAccount: raffle_treasury_account,
      })
      .rpc({
        skipPreflight: true,
      });

    console.log("Add Reward Tx", tx);
  });
  it("Should create the tree and mint a cnft", async () => {
    const { tree, collection } = await createAndMint();
    if (!tree.treeAddress) {
      throw new Error("Tree address not found");
    }
    treeAddress = tree.treeAddress;
    treeAuthority = tree.treeAuthority;

    const fetchcNFTs = await getcNFTsFromCollection(
      collection.mint,
      wallet.publicKey.toString()
    );

    cnftCollection = collection.mint;
    console.log("fetchcNFTs", fetchcNFTs);
    assetId = fetchcNFTs[0];
    console.log("assetId", assetId);
  });
  it("should create a entry", async () => {
    let [entry_account, entry_bump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("entry"),
          wallet.publicKey.toBuffer(),
          raffle_account.toBuffer(),
        ],
        program.programId
      );

    const asset = await getAsset(assetId);
    const proof = await getAssetProof(assetId);
    const root = decode(proof.root);
    const proofPathAsAccounts = mapProof(proof);
    const dataHash = decode(asset.compression.data_hash);
    const creatorHash = decode(asset.compression.creator_hash);
    const nonce = new anchor.BN(asset.compression.leaf_id);
    const index = asset.compression.leaf_id;
    const tx = await program.methods
      .createEntry(root, dataHash, creatorHash, nonce, index)
      .accounts({
        authority: wallet.publicKey,
        raffleAccount: raffle_account,
        cnftCollection: cnftCollection,
        merkleTree: treeAddress,
        ticketAccount: entry_account,
        bubblegumProgram: BUBBLEGUM_PROGRAM_ID,
        logWrapper: SPL_NOOP_PROGRAM_ID,
        compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts(proofPathAsAccounts)
      .rpc({
        skipPreflight: true,
      });
    console.log("Create Entry Tx", tx);
  });
});
