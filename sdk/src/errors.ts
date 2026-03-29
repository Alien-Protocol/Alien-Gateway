export class AlienGatewayError extends Error {
  public readonly cause?: unknown;

  public constructor(message: string, options?: { cause?: unknown }) {
    super(message);
    this.name = new.target.name;
    this.cause = options?.cause;
  }
}

export class UsernameUnavailableError extends AlienGatewayError {
  public constructor(
    public readonly username: string,
    public readonly commitment: string,
  ) {
    super(`Username "${username}" is already registered.`);
  }
}

export class ProofGenerationError extends AlienGatewayError {
  public constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
  }
}

export class TransactionFailedError extends AlienGatewayError {
  public constructor(
    message: string,
    public readonly txHash?: string,
    options?: { cause?: unknown },
  ) {
    super(message, options);
  }
}

export class UsernameNotFoundError extends AlienGatewayError {
  public constructor(public readonly username: string) {
    super(`Username "${username}" not found.`);
  }
}

export class NoAddressLinkedError extends AlienGatewayError {
  public constructor(public readonly username: string) {
    super(`Username "${username}" does not have a linked Stellar address.`);
  }
}
