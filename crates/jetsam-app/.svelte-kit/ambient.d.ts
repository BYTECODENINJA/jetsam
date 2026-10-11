
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/private';
 * 
 * console.log(ENVIRONMENT); // => "production"
 * console.log(PUBLIC_BASE_URL); // => throws error during build
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/private' {
	export const SVELTEKIT_FORK: string;
	export const NODE_ENV: string;
	export const OLDPWD: string;
	export const __MISE_ZSH_CHPWD_RAN: string;
	export const KITTY_INSTALLATION_DIR: string;
	export const MAIL: string;
	export const npm_config_global_prefix: string;
	export const AQ_NO_MODIFIERS: string;
	export const GDK_SCALE: string;
	export const PATH: string;
	export const npm_config_noproxy: string;
	export const GDK_BACKEND: string;
	export const XDG_DATA_DIRS: string;
	export const QT_AUTO_SCREEN_SCALE_FACTOR: string;
	export const QML_IMPORT_PATH: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const npm_package_json: string;
	export const XDG_RUNTIME_DIR: string;
	export const npm_execpath: string;
	export const QML2_IMPORT_PATH: string;
	export const npm_config_user_agent: string;
	export const XDG_SESSION_ID: string;
	export const PAGER: string;
	export const SHLVL: string;
	export const GSK_RENDERER: string;
	export const npm_lifecycle_event: string;
	export const VISUAL: string;
	export const HYPRLAND_INSTANCE_SIGNATURE: string;
	export const __MISE_SESSION: string;
	export const ZSH_CUSTOM: string;
	export const npm_config_prefix: string;
	export const __MISE_ZSH_PRECMD_RUN: string;
	export const ZSH: string;
	export const QT_QPA_PLATFORMTHEME: string;
	export const TERMINFO: string;
	export const PWD: string;
	export const XCURSOR_SIZE: string;
	export const RYOKU_POLKIT_AGENT: string;
	export const XCURSOR_THEME: string;
	export const npm_config_userconfig: string;
	export const HL_INITIAL_WORKSPACE_TOKEN: string;
	export const DESKTOP_SESSION: string;
	export const npm_config_local_prefix: string;
	export const npm_config_globalconfig: string;
	export const npm_config_npm_version: string;
	export const COLOR: string;
	export const XDG_VTNR: string;
	export const AGENT: string;
	export const FZF_ALT_C_COMMAND: string;
	export const GOBIN: string;
	export const HYPRCURSOR_SIZE: string;
	export const LOGNAME: string;
	export const KITTY_PID: string;
	export const npm_command: string;
	export const EDITOR: string;
	export const LSCOLORS: string;
	export const npm_config_global_ignore_file: string;
	export const DISPLAY: string;
	export const ELECTRON_OZONE_PLATFORM_HINT: string;
	export const NODE: string;
	export const FZF_DEFAULT_COMMAND: string;
	export const COLORTERM: string;
	export const XDG_SESSION_DESKTOP: string;
	export const npm_node_execpath: string;
	export const HYPRLAND_CMD: string;
	export const npm_config_init_module: string;
	export const npm_config_cache: string;
	export const PNPM_HOME: string;
	export const npm_config_node_gyp: string;
	export const SHELL: string;
	export const QT_WAYLAND_DISABLE_WINDOWDECORATION: string;
	export const XDG_SEAT_PATH: string;
	export const XDG_SESSION_TYPE: string;
	export const XDG_CURRENT_DESKTOP: string;
	export const _: string;
	export const KITTY_PUBLIC_KEY: string;
	export const CARGO_INSTALL_ROOT: string;
	export const USER: string;
	export const FZF_CTRL_T_COMMAND: string;
	export const OPENCODE_PID: string;
	export const HOME: string;
	export const XDG_BACKEND: string;
	export const OPENCODE: string;
	export const HYPRCURSOR_THEME: string;
	export const LANG: string;
	export const _JAVA_AWT_WM_NONREPARENTING: string;
	export const VSSCRIPT_PATH: string;
	export const LS_COLORS: string;
	export const XDG_SEAT: string;
	export const npm_package_version: string;
	export const npm_lifecycle_script: string;
	export const MOTD_SHOWN: string;
	export const STARSHIP_SHELL: string;
	export const XDG_SESSION_PATH: string;
	export const KITTY_WINDOW_ID: string;
	export const MOZ_ENABLE_WAYLAND: string;
	export const WAYLAND_DISPLAY: string;
	export const __MISE_DIFF: string;
	export const STARSHIP_SESSION_KEY: string;
	export const MISE_SHELL: string;
	export const QT_QPA_PLATFORM: string;
	export const DEBUGINFOD_URLS: string;
	export const INIT_CWD: string;
	export const __MISE_ORIG_PATH: string;
	export const XDG_SESSION_CLASS: string;
	export const TERM: string;
	export const npm_package_name: string;
}

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/public';
 * 
 * console.log(ENVIRONMENT); // => throws error during build
 * console.log(PUBLIC_BASE_URL); // => "http://site.com"
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * 
 * console.log(env.ENVIRONMENT); // => "production"
 * console.log(env.PUBLIC_BASE_URL); // => undefined
 * ```
 */
declare module '$env/dynamic/private' {
	export const env: {
		SVELTEKIT_FORK: string;
		NODE_ENV: string;
		OLDPWD: string;
		__MISE_ZSH_CHPWD_RAN: string;
		KITTY_INSTALLATION_DIR: string;
		MAIL: string;
		npm_config_global_prefix: string;
		AQ_NO_MODIFIERS: string;
		GDK_SCALE: string;
		PATH: string;
		npm_config_noproxy: string;
		GDK_BACKEND: string;
		XDG_DATA_DIRS: string;
		QT_AUTO_SCREEN_SCALE_FACTOR: string;
		QML_IMPORT_PATH: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		npm_package_json: string;
		XDG_RUNTIME_DIR: string;
		npm_execpath: string;
		QML2_IMPORT_PATH: string;
		npm_config_user_agent: string;
		XDG_SESSION_ID: string;
		PAGER: string;
		SHLVL: string;
		GSK_RENDERER: string;
		npm_lifecycle_event: string;
		VISUAL: string;
		HYPRLAND_INSTANCE_SIGNATURE: string;
		__MISE_SESSION: string;
		ZSH_CUSTOM: string;
		npm_config_prefix: string;
		__MISE_ZSH_PRECMD_RUN: string;
		ZSH: string;
		QT_QPA_PLATFORMTHEME: string;
		TERMINFO: string;
		PWD: string;
		XCURSOR_SIZE: string;
		RYOKU_POLKIT_AGENT: string;
		XCURSOR_THEME: string;
		npm_config_userconfig: string;
		HL_INITIAL_WORKSPACE_TOKEN: string;
		DESKTOP_SESSION: string;
		npm_config_local_prefix: string;
		npm_config_globalconfig: string;
		npm_config_npm_version: string;
		COLOR: string;
		XDG_VTNR: string;
		AGENT: string;
		FZF_ALT_C_COMMAND: string;
		GOBIN: string;
		HYPRCURSOR_SIZE: string;
		LOGNAME: string;
		KITTY_PID: string;
		npm_command: string;
		EDITOR: string;
		LSCOLORS: string;
		npm_config_global_ignore_file: string;
		DISPLAY: string;
		ELECTRON_OZONE_PLATFORM_HINT: string;
		NODE: string;
		FZF_DEFAULT_COMMAND: string;
		COLORTERM: string;
		XDG_SESSION_DESKTOP: string;
		npm_node_execpath: string;
		HYPRLAND_CMD: string;
		npm_config_init_module: string;
		npm_config_cache: string;
		PNPM_HOME: string;
		npm_config_node_gyp: string;
		SHELL: string;
		QT_WAYLAND_DISABLE_WINDOWDECORATION: string;
		XDG_SEAT_PATH: string;
		XDG_SESSION_TYPE: string;
		XDG_CURRENT_DESKTOP: string;
		_: string;
		KITTY_PUBLIC_KEY: string;
		CARGO_INSTALL_ROOT: string;
		USER: string;
		FZF_CTRL_T_COMMAND: string;
		OPENCODE_PID: string;
		HOME: string;
		XDG_BACKEND: string;
		OPENCODE: string;
		HYPRCURSOR_THEME: string;
		LANG: string;
		_JAVA_AWT_WM_NONREPARENTING: string;
		VSSCRIPT_PATH: string;
		LS_COLORS: string;
		XDG_SEAT: string;
		npm_package_version: string;
		npm_lifecycle_script: string;
		MOTD_SHOWN: string;
		STARSHIP_SHELL: string;
		XDG_SESSION_PATH: string;
		KITTY_WINDOW_ID: string;
		MOZ_ENABLE_WAYLAND: string;
		WAYLAND_DISPLAY: string;
		__MISE_DIFF: string;
		STARSHIP_SESSION_KEY: string;
		MISE_SHELL: string;
		QT_QPA_PLATFORM: string;
		DEBUGINFOD_URLS: string;
		INIT_CWD: string;
		__MISE_ORIG_PATH: string;
		XDG_SESSION_CLASS: string;
		TERM: string;
		npm_package_name: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://example.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.ENVIRONMENT); // => undefined, not public
 * console.log(env.PUBLIC_BASE_URL); // => "http://example.com"
 * ```
 * 
 * ```
 * 
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
