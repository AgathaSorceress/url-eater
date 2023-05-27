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
category "Campaign tracking (stm)" disabled=true {
	params "stm_*"
}
```
Categories are used to structure filter lists and allow disabling/enabling filters in groups. 
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

## Usage example
This repository also contains a Nix flake. It can be used in a NixOS configuration like this:  
1. Add flake to inputs:
```nix
url-eater.url = "github:AgathaSorceress/url-eater";
url-eater.inputs.nixpkgs.follows = "nixpkgs"; #optional
```
2. Adding output: 
```nix
  outputs = inputs@{ self, nixpkgs, url-eater, ... }:
```
3. Import NixOS module
```nix
imports = [ url-eater.nixosModule ];
```
4. Configure the module:
```nix
{ ... }: {
  services.url-eater = {
    enable = true;
    filters = ''
      category "Spotify" {
      	params "context@open.spotify.com" "si@open.spotify.com"
      }
      category "Twitter" {
      	params "cxt@*.twitter.com" "ref_*@*.twitter.com" "s@*.twitter.com" "t@*.twitter.com" "twclid"
      }
    '';
  };
}
```

## Building from source
Clone this repository, then run:
```sh
cargo build --release
```
You will need Rust 1.65 or newer.  
The output binary will be in `target/release/url-eater`  

Alternatively,
```sh
nix build github:AgathaSorceress/url-eater
```
