import Foundation

public struct ChainPrivateKey {
    public var contents: String
    public var publicKey: ChainPublicKey

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        contents: String,
        publicKey: ChainPublicKey) {
        self.contents = contents
        self.publicKey = publicKey
    }
}


extension ChainPrivateKey: Equatable, Hashable {
    public static func ==(lhs: ChainPrivateKey, rhs: ChainPrivateKey) -> Bool {
        if lhs.contents != rhs.contents {
            return false
        }
        if lhs.publicKey != rhs.publicKey {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(contents)
        hasher.combine(publicKey)
    }
}

public struct ChainPublicKey {
    public var contents: String
    public var chain: Blockchain

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        contents: String,
        chain: Blockchain) {
        self.contents = contents
        self.chain = chain
    }
}


extension ChainPublicKey: Equatable, Hashable {
    public static func ==(lhs: ChainPublicKey, rhs: ChainPublicKey) -> Bool {
        if lhs.contents != rhs.contents {
            return false
        }
        if lhs.chain != rhs.chain {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(contents)
        hasher.combine(chain)
    }
}

public struct ChainTransaction {
    public var tx: String
    public var signers: [ChainPublicKey]
    public var fullSignature: String?
    public var signatures: [String]?
    public var accounts: [ChainPublicKey]
    public var instructionPrograms: [String]

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        tx: String,
        signers: [ChainPublicKey],
        fullSignature: String?,
        signatures: [String]?,
        accounts: [ChainPublicKey],
        instructionPrograms: [String]) {
        self.tx = tx
        self.signers = signers
        self.fullSignature = fullSignature
        self.signatures = signatures
        self.accounts = accounts
        self.instructionPrograms = instructionPrograms
    }
}

extension ChainTransaction: Equatable, Hashable {
    public static func ==(lhs: ChainTransaction, rhs: ChainTransaction) -> Bool {
        if lhs.tx != rhs.tx {
            return false
        }
        if lhs.signers != rhs.signers {
            return false
        }
        if lhs.fullSignature != rhs.fullSignature {
            return false
        }
        if lhs.signatures != rhs.signatures {
            return false
        }
        if lhs.accounts != rhs.accounts {
            return false
        }
        if lhs.instructionPrograms != rhs.instructionPrograms {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(tx)
        hasher.combine(signers)
        hasher.combine(fullSignature)
        hasher.combine(signatures)
        hasher.combine(accounts)
        hasher.combine(instructionPrograms)
    }
}

public struct DecimalNumber {
    public var value: String

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        value: String) {
        self.value = value
    }
}


extension DecimalNumber: Equatable, Hashable {
    public static func ==(lhs: DecimalNumber, rhs: DecimalNumber) -> Bool {
        if lhs.value != rhs.value {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(value)
    }
}

public struct Derivation {
    public var start: UInt32
    public var count: UInt32
    public var path: DerivationPath

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        start: UInt32,
        count: UInt32,
        path: DerivationPath) {
        self.start = start
        self.count = count
        self.path = path
    }
}


extension Derivation: Equatable, Hashable {
    public static func ==(lhs: Derivation, rhs: Derivation) -> Bool {
        if lhs.start != rhs.start {
            return false
        }
        if lhs.count != rhs.count {
            return false
        }
        if lhs.path != rhs.path {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(start)
        hasher.combine(count)
        hasher.combine(path)
    }
}

public struct DerivedPrivateKey {
    public var contents: String
    public var publicKey: ChainPublicKey
    public var index: UInt32
    public var path: String?
    public var pathType: DerivationPath?

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        contents: String,
        publicKey: ChainPublicKey,
        index: UInt32,
        path: String?,
        pathType: DerivationPath?) {
        self.contents = contents
        self.publicKey = publicKey
        self.index = index
        self.path = path
        self.pathType = pathType
    }
}


extension DerivedPrivateKey: Equatable, Hashable {
    public static func ==(lhs: DerivedPrivateKey, rhs: DerivedPrivateKey) -> Bool {
        if lhs.contents != rhs.contents {
            return false
        }
        if lhs.publicKey != rhs.publicKey {
            return false
        }
        if lhs.index != rhs.index {
            return false
        }
        if lhs.path != rhs.path {
            return false
        }
        if lhs.pathType != rhs.pathType {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(contents)
        hasher.combine(publicKey)
        hasher.combine(index)
        hasher.combine(path)
        hasher.combine(pathType)
    }
}

public struct ExternalAddress {
    public var recentBlockhash: String

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        recentBlockhash: String) {
        self.recentBlockhash = recentBlockhash
    }
}


extension ExternalAddress: Equatable, Hashable {
    public static func ==(lhs: ExternalAddress, rhs: ExternalAddress) -> Bool {
        if lhs.recentBlockhash != rhs.recentBlockhash {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(recentBlockhash)
    }
}

public struct MnemonicWords {
    public var words: [String]

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        words: [String]) {
        self.words = words
    }
}

extension MnemonicWords: Equatable, Hashable {
    public static func ==(lhs: MnemonicWords, rhs: MnemonicWords) -> Bool {
        if lhs.words != rhs.words {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(words)
    }
}

public struct ParsedTransaction {
    public var from: ChainPublicKey?
    public var to: ChainPublicKey
    public var data: TransactionData

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        from: ChainPublicKey?,
        to: ChainPublicKey,
        data: TransactionData) {
        self.from = from
        self.to = to
        self.data = data
    }
}


extension ParsedTransaction: Equatable, Hashable {
    public static func ==(lhs: ParsedTransaction, rhs: ParsedTransaction) -> Bool {
        if lhs.from != rhs.from {
            return false
        }
        if lhs.to != rhs.to {
            return false
        }
        if lhs.data != rhs.data {
            return false
        }
        return true
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(from)
        hasher.combine(to)
        hasher.combine(data)
    }
}

// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum Blockchain {

    case solana
}

extension Blockchain: Equatable, Hashable {}

// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum DerivationPath {

    case bip44Root
    case bip44
    case bip44Change
    case deprecated
}

extension DerivationPath: Equatable, Hashable {}



public enum EncryptionError {
    case Generic(message: String)
}


extension EncryptionError: Equatable, Hashable {}

extension EncryptionError: Error { }


public enum KeyError {



    case InvalidKeypair(message: String)

    case InvalidMnemonic(message: String)

    case DerivationPath(message: String)

    case PrivateKey(message: String)

    case PublicKey(message: String)

    case Generic(message: String)
}


extension KeyError: Equatable, Hashable {}

extension KeyError: Error { }

// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum SolanaTransactionType {

    case legacy
    case versioned
}

extension SolanaTransactionType: Equatable, Hashable {}


// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum TokenDestination {

    case account(
        transferDestination: String
    )
    case wallet(
        publicKey: ChainPublicKey
    )
}

extension TokenDestination: Equatable, Hashable {}



// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum TransactionData {

    case solana
}

extension TransactionData: Equatable, Hashable {}




public enum TransactionError {



    case KeyPair(message: String)

    case SignerMissing(message: String)

    case MultipleSigners(message: String)

    case PrivateKey(message: String)

    case PublicKey(message: String)

    case Parameters(message: String)

    case ParsingFailure(message: String)

    case InstructionError(message: String)

    case DecimalConversion(message: String)

    case SignMsgError(message: String)

    case Generic(message: String)
}

extension TransactionError: Equatable, Hashable {}

extension TransactionError: Error { }

// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum TransactionKind {

    case token(
        amount: DecimalNumber,
        closeAccount: Bool
    )
    case nft(
        amount: UInt64,
        id: String?
    )
}

extension TransactionKind: Equatable, Hashable {}

// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum TransactionParameters {

    case solana(
        externalAddress: ExternalAddress?,
        transactionType: SolanaTransactionType,
        ownerProgram: String?,
        decimals: UInt8?,
        memo: String?,
        references: [String],
        swapSlippageBps: UInt16?,
        computeBudgetUnitPrice: UInt64?,
        computeBudgetUnitLimit: UInt32?
    )
}

extension TransactionParameters: Equatable, Hashable {}

public func appendSignatureToTransaction(signer: String, signature: String, transaction: String) throws  -> String {
    ""
}
public func decryptCiphertext(ciphertext: String, password: String) throws  -> String {
    ""
}
public func derive(
    chain: Blockchain,
    mnemonic: MnemonicWords,
    passphrase: String?,
    derivation: Derivation
) throws  -> [DerivedPrivateKey] {
    return []
}

public func deriveFromData(chain: Blockchain, data: String) throws  -> DerivedPrivateKey {
    DerivedPrivateKey(
        contents: "",
        publicKey: ChainPublicKey(contents: "", chain: chain),
        index: 0,
        path: nil,
        pathType: nil
    )
}

public func encryptPlaintext(plaintext: String, password: String) throws  -> String {
    ""
}

public func generateMnemonic(length: UInt32) throws  -> MnemonicWords {
    MnemonicWords(words: [])
}

public func getAssociatedTokenAddress(walletAddress: String, ownerProgram: String, tokenMintAddress: String) throws  -> ChainPublicKey {
    ChainPublicKey(
        contents: "",
        chain: .solana
    )
}

public func getMessage(transaction: String) throws  -> String {
    ""
}

public func getProgramAddress(seeds: [String], program: String) throws  -> ChainPublicKey {
    ChainPublicKey(
        contents: "",
        chain: .solana
    )
}

public func isValid(chain: Blockchain, address: String)  -> Bool {
    true
}

public func modifyTransaction(chain: Blockchain, transaction: String, owner: ChainPrivateKey, parameters: TransactionParameters) throws  -> String {
    transaction
}
public func parsePrivateKey(key: String)  -> ChainPrivateKey? {
    nil
}
public func parsePublicKey(address: String)  -> ChainPublicKey? {
    nil
}
public func parseTransaction(chain: Blockchain, transaction: String) throws  -> ParsedTransaction {
    ParsedTransaction(
        from: nil,
        to: ChainPublicKey(contents: "", chain: chain),
        data: .solana
    )
}
public func rawPrivateKey(chain: Blockchain, key: String) throws  -> ChainPrivateKey {
    ChainPrivateKey(
        contents: key,
        publicKey: ChainPublicKey(contents: key, chain: chain)
    )
}
public func sendTransaction(chain: Blockchain, sender: ChainPublicKey, receiver: ChainPublicKey, amount: DecimalNumber, parameters: TransactionParameters) throws  -> String {
    ""
}
public func signMessage(chain: Blockchain, message: String, signers: [ChainPrivateKey]) throws  -> String {
    ""
}
public func signTransaction(chain: Blockchain, transaction: String, signers: [ChainPrivateKey], parameters: TransactionParameters?) throws  -> ChainTransaction {
    ChainTransaction(
        tx: transaction,
        signers: signers.map { $0.publicKey },
        fullSignature: nil,
        signatures: nil,
        accounts: [],
        instructionPrograms: []
    )
}
public func signTypedData(chain: Blockchain, typedData: String, signers: [ChainPrivateKey]) throws  -> String {
    ""
}
public func tokenTransaction(chain: Blockchain, destination: TokenDestination, owner: ChainPublicKey, token: ChainPublicKey, kind: TransactionKind, parameters: TransactionParameters) throws  -> String {
    ""
}
