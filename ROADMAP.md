# Roadmap and check list

## Stage 1 (Basic hashchain of records)
- [x] ability to add records and hash them
- [x] ability to verify a record using cryptography
- [x] 'evil modify' function that modifies data and we test that the verification does work

## Stage 2 (Users and signatures)
- [x] ability for users to sign a record
- [x] ability to verify the signatures too


## Stage 3 (Workflows, States, Rules) – hardest so far
- [ ] define entity states
- [ ] define allowed state transition
- [ ] enforce that records cannot skip or reverse states unless alowed
- [ ] workflow engine validations a transition before adding a record
- [ ] role based approval rules per transition
- [ ] workflow engine shld load rules from .yaml
- [ ] derive current state from latest valid record
- [ ] detect invalid workflow attempts before they hit the chain



## Extras
- [ ] timestamps and make sure they are well validated
- [ ] persitence? binary or json? if so they should be well compressed
- [ ] if time allows make this distributed? very hard but should be possible