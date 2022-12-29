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
  - [ ] Bitcoin or Lightning Bitcoin payment 1 time or recurrent to be in the allowlist

## API's

### For server that requires authentication

You can only make one authenticated request with the same signature.

| Parameter | Type | Description | Example | Is required? |
| --- | --- | --- | --- | --- |
| pubkey | string | Your public key | `884704bd421721e292edbff42eb77547fe115c6ff9825b08fc366be4cd69e9f6` | yes |
| uniq | string | a unique (random) string | `20` | yes |
| time | number | Unix timestamp (UTC-0) | `1600000000` | yes |
| sig | string | Signature of: `sha256(string: "{pubkey}:{time}:{uniq}"")` | `0ae1feeb6fb36f3f5f5d3f001b06a5f6d01c999d7a74b9227012cdac0587f1ef7b9ed4b5e16afd3f1f502266f0b3b2ed21906554d6e4ffba43de2bb99d061694` | yes |
| sig | string | pubkey | `884704bd421721e292edbff42eb77547fe115c6ff9825b08fc366be4cd69e9f6` | yes |

### GET /nip05

Example without Authentification required: `https://example.com/nip05?nip05=_@nostr.0xtlt.dev`
Example with Authentification required: `https://example.com/nip05?nip05=_@nostr.0xtlt.dev&pubkey=884704bd421721e292edbff42eb77547fe115c6ff9825b08fc366be4cd69e9f6&uniq=20&time=1600000000&sig=0ae1feeb6fb36f3f5f5d3f001b06a5f6d01c999d7a74b9227012cdac0587f1ef7b9ed4b5e16afd3f1f502266f0b3b2ed21906554d6e4ffba43de2bb99d061694`

| Parameter | Type | Description | Example | Is required? |
| --- | --- | --- | --- | --- |
| nip05 | string | NIP-05 to load | `_@nostr.0xtlt.dev` | yes |


#### Response type

```ts
type NIP05Response = {
  status: "error";
  message: string;
} | {
  pubkey: string;
  status: "success";
  updated_at: number;
}
```

### GET /image_proxy

Coming...

### GET /website_preview

Coming...
