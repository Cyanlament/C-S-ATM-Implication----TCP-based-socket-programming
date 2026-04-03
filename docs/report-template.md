# Lab2 Report Template (ATM over TCP)

## 1. Basic Info

- Student name:
- Student ID:
- Group ID:
- Partner:
- Course week:
- GitHub repository link:

## 2. Requirement Mapping (from PPT)

- [ ] Use TCP socket programming
- [ ] Follow RFC-20232023 protocol messages
- [ ] Client has GUI
- [ ] Server has no GUI
- [ ] Server reads account data file
- [ ] Server logs all exceptions
- [ ] Server logs all withdrawal records
- [ ] Default port is 2525
- [ ] Client provides complete test case to visit other group server

## 3. Protocol Design

Request messages:
- HELO <userid>
- PASS <passwd>
- BALA
- WDRA <amount>
- BYE

Response messages:
- 500 AUTH REQUIRED!
- 525 OK!
- 401 ERROR!
- AMNT:<amnt>
- BYE

## 4. System Design

### 4.1 Rust version

- Server entry:
- Client GUI entry:
- Data file path:
- Log file paths:

### 4.2 TypeScript version

- Server entry:
- Client GUI entry:
- Data file path:
- Log file paths:

## 5. Test Cases

### 5.1 Normal flow
- HELO valid user -> 500 AUTH REQUIRED!
- PASS valid password -> 525 OK!
- BALA -> AMNT:<value>
- WDRA valid amount with enough balance -> 525 OK!
- BYE -> BYE

### 5.2 Error flow
- PASS before HELO -> 401 ERROR!
- Invalid user in HELO -> 401 ERROR!
- Wrong password in PASS -> 401 ERROR!
- WDRA with insufficient funds -> 401 ERROR!
- Invalid request format -> 401 ERROR!

### 5.3 Cross-group server test

- Target group server host:
- Target group server port:
- Steps and responses:
- Result:

## 6. Run Commands

### Rust
- Server:
- GUI client:
- Test script:

### TypeScript
- Install:
- Server:
- GUI client:
- Test script:

## 7. Logs and Evidence

Attach screenshots or snippets for:
- GUI client interaction
- exception.log
- withdraw.log
- Updated account data after withdrawal

## 8. Problems and Fixes

- Issue 1:
- Root cause:
- Fix:

## 9. Conclusion

- What is completed:
- Remaining work:
- Personal reflection:
