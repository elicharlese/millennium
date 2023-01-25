<div align=center>
	<a href="https://millennium.pyke.io"><img src="https://github.com/pykeio/millennium/raw/main/.github/banner.png" width=1920></a>
	<br /><br />
	<a href="https://github.com/pykeio/millennium/actions"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/pykeio/millennium/test-main.yml?branch=main&style=for-the-badge&logo=github-actions&logoColor=white"></a>  <a href="https://github.com/pykeio/millennium/actions"><img alt="Audit Status" src="https://img.shields.io/github/actions/workflow/status/pykeio/millennium/audit.yml?branch=main&style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZmlsbD0iI2ZmZiIgZD0iTTEyIDEyaDdjLS41IDQuMS0zLjMgNy44LTcgOXYtOUg1VjYuM2w3LTMuMU0xMiAxIDMgNXY2YzAgNS42IDMuOCAxMC43IDkgMTIgNS4yLTEuMyA5LTYuNCA5LTEyVjVsLTktNFoiLz48L3N2Zz4%3D&label=audit"></a> <a href="https://crates.io/crates/millennium"><img alt="Crates.io" src="https://img.shields.io/crates/d/millennium?style=for-the-badge&logo=rust"></a> <a href="https://discord.gg/BAkXJ6VjCz"><img alt="Discord" src="https://img.shields.io/discord/1029216970027049072?style=for-the-badge&logo=discord&logoColor=white"></a>
	<br /><br />
	<hr />
</div>

Millennium is a cross-platform webview framework written in Rust. With Millennium, you can design consistent UI that works across all platforms, using HTML, CSS, and JavaScript.

## How it works
Millennium leverages the webview API pre-installed on modern desktop & mobile operating systems for smaller, faster applications compared to Electron or NW.js. With Millennium's JavaScript API, you can interact with native Rust code and perform system-level operations, including reading/writing files & networking. A simple Millennium app can be less than **10 MB** in size and can be reduced further to less than **2 MB**. Millennium apps can launch almost twice as fast as equivalent Electron applications and use as little as __1/4 of the amount of RAM.__

Millennium is a fork of [Tauri](https://tauri.app/), [`tao`](https://github.com/tauri-apps/tao/), [`wry`](https://github.com/tauri-apps/wry), and some official plugins, with a few added features and changes ✨

## `@pyke/millennium-api`
This package provides the JavaScript API for [Millennium](https://millennium.pyke.io/) applications. With `@pyke/millennium-api`, you can create windows, execute processes, read files, and more from JavaScript with APIs similar to (but not 100% compatible with) Node.js.

You can also access the Millennium API without this module via `window.Millennium` if `build > withGlobalMillennium` is enabled in your Millennium config file. We highly recommend that you bundle this module with your frontend code for production apps for security.

## Learn more

- [**More information**](https://millennium.pyke.io/)
- [**Getting started**](https://millennium.pyke.io/docs/main/your-first-app/prerequisites)
- [**Rust API reference**](https://docs.rs/millennium)
- [**Discord server**](https://discord.gg/BAkXJ6VjCz)
