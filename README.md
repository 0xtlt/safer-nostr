# safer-nostr

Safer Nostr is a service that helps protect users by loading sensitive information (IP leak) and using AI to prevent inappropriate images from being uploaded. It also offers image optimization and storage options. It has configurable privacy and storage settings, as well as custom cache expiration.

## Key features

- [x] Load NIP-05
- [x] Load website preview
- Cache non image files
  - [x] Cache in Redis
  - [ ] Cache in RAM
- [x] Load and optimize images
  - [x] Store images in Redis
  - [ ] Store images in RAM
  - [ ] Store images in S3
  - [ ] Store images in local disk
  - [ ] Artificial intelligence checks for inappropriate images
- [x] Configurable settings
  - [x] Private or public mode
  - [x] RAM or Redis storage options
  - [x] Custom cache expiration time
