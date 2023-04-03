# Image Scaling Service

## Env Variables
- DOMAINS: ; seperated list of urls (https://example.com;http://test.xyz...)
- MODE: whitelist/blacklist

## Query Params
- url: source of the image (required)
- width
- height
- format: output format (default png)
- cache_max_age: sets the Cache-Control header on the image
