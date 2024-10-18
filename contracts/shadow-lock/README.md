# Delegate Lock

## RFC

`Delegate Lock` is a generic lock in order to create a proxy ownership control to specified cell’s type or lock.

The core concept of `Delegate Lock` is to simplify the process of composing different cells on-chain. Also provides a generic way for ownership delegating on-chain.

### Data Structure

A cell that applies `Delegate Lock` should like following:

```yaml
Lock:
  hash_type: "data1"
  code_hash: Delegate_LOCK_CODE_HASH
  args: <mode flags, 1byte><delegate script hash, 32 bytes>[<delegate data hash, 32bytes,optional>]
```

#### mode flags

mode flags is a bitset flag map that toggles different features. Different feature bits can be set individually or together.

| Name | Flags | Affected Args | Affected Behavior  |
| --- | --- | --- | --- |
| delegate script type | 0b00000001 | Delegate Script Hash | If set to 1, then delegate target will be Type script, otherwise will be Lock script |
| forbid trade | 0b00000010 | N/A | If set to 1, the lock can only be unlock once and can not be set in output again(but can set to different args if using same lock script) |
| self destruction | 0b00000100 | N/A | If set to 1, this cell must be destroyed after an unlock |
| restrict delegate data | 0b00001000 | N/A | if set to 1, the optional 32 bytes of data hash in args must be set. then you will need a matching data of the cell in order to unlock |

### Operations

This section describes operations and restrictions in Delegate Lock implementation

#### Delegate/Compose

Delegate/compose is setting one or more cell’s lock from other lock to Delegate Lock.

```yaml
// Delegate/Compose
Inputs:
  <...>
  Cell_1:
    Type: <USER_DEFINED>
    Lock: <USER_DEFINED>
  Delegate Cell:
    Type: <USER_DEFINED> # hash = TYPE_HASH_1
    Lock: <USER_DEFINED> # hash = LOCK_HASH_1
  <...>
Outputs:
  <...>
  Cell_1:
    Type: <USER_DEFINED>
    Lock:
      hash_type: "data1"
      code_hash: Delegate_LOCK_CODE_HASH
      args: <0b00000111, 1byte><TYPE_HASH_1, 32bytes>
  Delegate Cell:
    Type: <USER_DEFINED> # hash = TYPE_HASH_1
    Lock: <USER_DEFINED> # hash = LOCK_HASH_1
  <...>
```

#### Decompose/Unlock

Decompose/Unlock is setting one or more cell’s lock from Delegate Lock to others.

```yaml
// Delegate/Compose
Inputs:
  <...>
  Cell_1:
    Type: <USER_DEFINED>
    Lock:
      hash_type: "data1"
      code_hash: Delegate_LOCK_CODE_HASH
      args: <0b00000111, 1byte><TYPE_HASH_1, 32bytes>
  Delegate Cell:
    Type: <USER_DEFINED> # hash = TYPE_HASH_1
    Lock: <USER_DEFINED> # hash = LOCK_HASH_1
  <...>
Outputs:
  <...>
  Cell_1:
    Type: <USER_DEFINED>
    Lock: <USER_DEFINED>
  Delegate Cell:
    Type: <USER_DEFINED> # hash = TYPE_HASH_1
    Lock: <USER_DEFINED> # hash = LOCK_HASH_1
  <...>
```

Limitation rules are described as below:

- The delegated cell must be present in inputs together while unlocking a `Delegate Lock`
- If `self destroy` mode flag it set to be true, then the Cell using Delegate lock must be destroyed while unlock
