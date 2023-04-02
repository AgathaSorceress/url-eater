# URL Eater
Clean unnecessary parameters from URLs copied to clipboard

## Usage
Run with a filter file that specifies which parameters should be removed:  
```
url-eater denylist.kdl
```
An example filter file:  
```d
category "Spotify" {
	params "context@open.spotify.com" "si@open.spotify.com"
}
category "Campaign tracking (itm)" {
	params "itm_*"
}
```
Categories do not have significance other than to make filter files better structured. 
Each parameter applies to all URLs, unless a domain like `@example.com` is specified at the end. 
Both the parameter and the domain parts can contain wildcards. Use `*` to match 0 or more characters, and `?` to match exactly one character.  
The structure is based on [NeatURL's format](https://github.com/Smile4ever/Neat-URL/#default-blocked-parameters), with a few differences (aside from a different file format):  
- Single character matching (`?`) is supported.  
- `$` and `!` rules are currently unsupported.  

The intended use case is running the program as a background service.

## Example
Before:
```
https://open.spotify.com/track/0ibuggkWTSDXHo25S0Qqvj?si=e4c675cbaee94c3a
```
After:
```
https://open.spotify.com/track/0ibuggkWTSDXHo25S0Qqvj
```