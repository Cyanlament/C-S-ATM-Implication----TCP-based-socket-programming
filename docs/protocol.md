# RFC-20232023 ATM Protocol (from lecture 08 PPT)

This document is transcribed from the teacher PPT (slides 1-3 screenshots) and used as the single source of truth for both Rust and TypeScript implementations.

## Transport

- TCP socket
- Default server port: 2525
- Text protocol, one message per line, UTF-8
- Message separator: LF (`\n`)

## Request Messages (ATM client -> Server)

1. `HELO <userid>`
- Purpose: ATM tells server there is a card inserted and sends card/user ID.

2. `PASS <passwd>`
- Purpose: User enters PIN/password and ATM sends it to server.

3. `BALA`
- Purpose: Request account balance.

4. `WDRA <amount>`
- Purpose: Request withdrawal.

5. `BYE`
- Purpose: End current ATM session.

## Response Messages (Server -> ATM client)

1. `500 AUTH REQUIRED!`
- Purpose: Ask for password/PIN after valid user identification.

2. `525 OK!`
- Purpose: Requested operation is successful (password check, withdrawal success).

3. `401 ERROR!`
- Purpose: Requested operation failed (invalid input, auth failed, insufficient funds, invalid state).

4. `AMNT:<amnt>`
- Purpose: Sent in response to balance request.

5. `BYE`
- Purpose: Session closed.

## Interaction Sequence

1. Client sends `HELO <userid>`
- Server checks if user ID is valid.
- If valid: server replies `500 AUTH REQUIRED!`
- If invalid: server replies `401 ERROR!`

2. Client sends `PASS <passwd>`
- Server checks password.
- If valid: server replies `525 OK!`
- If invalid: server replies `401 ERROR!`

3. Client may send `BALA`
- Server checks database and replies `AMNT:<amnt>`.

4. Client may send `WDRA <amount>`
- Server checks if balance is enough.
- If enough: update database and reply `525 OK!`.
- Else: reply `401 ERROR!`.

5. Client sends `BYE`
- Server replies `BYE` and closes session.

## Logging Requirements (from PPT slide 4)

Server must:
- Read data file
- Record logs for all exceptions
- Record logs for all withdrawal operations

Client side:
- Must provide GUI

Server side:
- GUI is not required
