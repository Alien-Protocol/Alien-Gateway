import {
  Address,
  BASE_FEE,
  Contract,
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  nativeToScVal,
  type xdr,
} from "@stellar/stellar-sdk";

import type {
  AddStellarAddressParams,
  BinaryInput,
  BuiltTransaction,
  Bytes32Input,
  PublicSignalsInput,
  RegisterParams,
  RegisterResolverParams,
  StellarTxBuilderConfig,
  SubmitTransactionOptions,
  TxBuildOptions,
} from "./types";

const DEFAULT_TIMEOUT_SECONDS = 60;

export class StellarTxBuilder {
  private readonly server: SorobanRpc.Server;
  private readonly contract: Contract;

  public constructor(private readonly config: StellarTxBuilderConfig) {
    this.server = new SorobanRpc.Server(config.rpcUrl, {
      allowHttp: config.allowHttp ?? isHttpUrl(config.rpcUrl),
    });
    this.contract = new Contract(config.contractAddress);
  }

  public async buildRegister(params: RegisterParams): Promise<BuiltTransaction> {
    return this.buildPreparedTransaction("register", [
      toScAddress(params.caller),
      toScBytes32(params.commitment),
    ], params);
  }

  public async buildRegisterResolver(params: RegisterResolverParams): Promise<BuiltTransaction> {
    return this.buildPreparedTransaction("register_resolver", [
      toScAddress(params.caller),
      toScBytes32(params.commitment),
      toScBytes(params.proof),
      toScPublicSignals(params.publicSignals),
    ], params);
  }

  public async buildAddStellarAddress(params: AddStellarAddressParams): Promise<BuiltTransaction> {
    return this.buildPreparedTransaction("add_stellar_address", [
      toScAddress(params.caller),
      toScBytes32(params.usernameHash),
      toScAddress(params.stellarAddress),
    ], params);
  }

  public async buildResolve(usernameHash: Bytes32Input, options: TxBuildOptions = {}): Promise<BuiltTransaction> {
    return this.buildPreparedTransaction("resolve_stellar", [toScBytes32(usernameHash)], options);
  }

  public async submitTransaction(
    built: BuiltTransaction | string,
    signer: Keypair | string,
    _options: SubmitTransactionOptions = {},
  ) {
    const signed = typeof built === "string"
      ? TransactionBuilder.fromXDR(built, this.config.networkPassphrase)
      : TransactionBuilder.fromXDR(built.xdr, this.config.networkPassphrase);
    const keypair = typeof signer === "string" ? Keypair.fromSecret(signer) : signer;

    signed.sign(keypair);

    return this.server.sendTransaction(signed);
  }

  private async buildPreparedTransaction(
    method: BuiltTransaction["method"],
    args: xdr.ScVal[],
    options: TxBuildOptions & { caller?: string },
  ): Promise<BuiltTransaction> {
    const source = resolveSource(options, this.config.defaultSource);
    if (!source) {
      throw new Error(`A source account is required to build ${method} transactions`);
    }

    const account = await this.server.getAccount(source);
    const raw = new TransactionBuilder(account, {
      fee: options.fee ?? this.config.defaultFee ?? BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(options.timeoutInSeconds ?? this.config.timeoutInSeconds ?? DEFAULT_TIMEOUT_SECONDS)
      .build();

    const prepared = await this.server.prepareTransaction(raw);

    return {
      transaction: prepared,
      xdr: prepared.toXDR(),
      method,
      source,
    };
  }
}

function toScAddress(address: string): xdr.ScVal {
  return new Address(address).toScVal();
}

function toScBytes32(value: Bytes32Input): xdr.ScVal {
  const bytes = normalizeBytes(value);
  if (bytes.length !== 32) {
    throw new Error(`Expected 32 bytes, received ${bytes.length}`);
  }

  return nativeToScVal(bytes, { type: "bytes" });
}

function toScBytes(value: BinaryInput): xdr.ScVal {
  return nativeToScVal(normalizeBytes(value), { type: "bytes" });
}

function toScPublicSignals(value: PublicSignalsInput): xdr.ScVal {
  return nativeToScVal({
    old_root: normalizeBytes32(value.oldRoot),
    new_root: normalizeBytes32(value.newRoot),
  });
}

function normalizeBytes32(value: Bytes32Input): Buffer {
  const bytes = normalizeBytes(value);
  if (bytes.length !== 32) {
    throw new Error(`Expected 32 bytes, received ${bytes.length}`);
  }

  return bytes;
}

function normalizeBytes(value: string | Uint8Array): Buffer {
  if (typeof value !== "string") {
    return Buffer.from(value);
  }

  const normalized = value.startsWith("0x") ? value.slice(2) : value;
  if (normalized.length % 2 === 0 && /^[0-9a-fA-F]+$/.test(normalized)) {
    return Buffer.from(normalized, "hex");
  }

  return Buffer.from(value, "base64");
}

function isHttpUrl(url: string): boolean {
  return url.startsWith("http://");
}

function resolveSource(
  options: TxBuildOptions & { caller?: string },
  defaultSource?: string,
): string | undefined {
  return options.source ?? options.caller ?? defaultSource;
}
