<div align="center">
	<h1>Bing Wallpaper for macOS</h1>
	<p>
		<b>Get daily wallpapers from Bing</b>
	</p>
	<br>
	<br>
	<br>
</div>

An application that updates your background every day and includes a collection of beautiful images from around the world that have been featured on the Bing homepage.

## Building

Make sure you are using the latest version of stable rust by running `rustup update`.

First install `cargo-bundle` to create a macOS Application Bundle:

```bash
  cargo install cargo-bundle
```

Then create the bundle using this command:
```bash
  cargo bundle --release
```
