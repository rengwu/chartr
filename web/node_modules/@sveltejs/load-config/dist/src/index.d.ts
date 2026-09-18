type ConfigSource = 'svelte' | 'vite';
interface SvelteConfig {
    compilerOptions?: Record<string, unknown>;
    preprocess?: unknown;
    extensions?: string[];
    kit?: unknown;
    vitePlugin?: unknown;
    [key: string]: unknown;
}
interface LoadedConfig {
    config: SvelteConfig;
    configFilePath: string;
    configSource: ConfigSource;
}
interface FailedConfig {
    error: unknown;
    configFilePath: string;
    configSource: ConfigSource;
}
type LoadConfigResult = LoadedConfig | FailedConfig | undefined;
/**
 * Loads the Svelte configuration by searching for `vite.config` and `svelte.config` files.
 *
 * If `dirOrFile` is a file path, that config file is loaded directly.
 *
 * If `dirOrFile` is a directory and `traverse` is true, it starts from the provided directory and traverses up the directory tree until it finds a config or reaches the root.
 * Else it only checks the provided directory.
 *
 * `vite.config` with either vite-plugin-svelte or the SvelteKit plugin providing options is preferred over `svelte.config`.
 *
 * The results are cached to optimize subsequent calls.
 */
export declare function loadConfig(dirOrFile: string, { traverse, clearCache }?: {
    traverse?: boolean;
    clearCache?: boolean;
}): Promise<LoadConfigResult>;
export {};
