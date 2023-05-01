# README

``RUST_LOG=debug cargo r``

Example of idempotent create operation (user registration) in an API layer, with [main logic here](/src/middleware/cache/mod.rs). It is best practice to have all ``POST`` endpoints accept an optional ``Idempotency-Key`` header in order to be a good distributed citizen. 

See [stripe/docs/api/idempotent_requests](https://stripe.com/docs/api/idempotent_requests)  and [draft-ietf-httpapi-idempotency-key-header/](https://datatracker.ietf.org/doc/draft-ietf-httpapi-idempotency-key-header/) for more details.

## Idempotency

Idempotency Key is a key provided by client in the headers that ensures API operations with side-effects (any ``POST`` endpoint) are idempotent, that is, run exactly once.

- [x] Given key ``K`` & user ``U`` for the first time: ``200`` & update cache.
- [x] Given key ``K`` & user ``U`` is repeated -> ``201``.
- [x] Given key ``K`` & user ``V`` -> *Depends of the Business Logic*.
- [x] Given key ``E`` and user ``S`` for the first time: -> ``200`` & update cache.
- [x] Given key ``E`` and user ``U`` -> ``409``.

## TODO

- [ ] Improve error handling.
- [ ] Cache errors.
