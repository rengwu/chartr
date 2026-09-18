"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadConfig = loadConfig;
const node_fs_1 = __importDefault(require("node:fs"));
const node_path_1 = __importDefault(require("node:path"));
const node_url_1 = require("node:url");
const VITE_CONFIG_EXTENSIONS = ['js', 'mjs', 'ts', 'cjs', 'mts', 'cts'];
const SVELTE_CONFIG_EXTENSIONS = ['js', 'cjs', 'mjs'];
const SVELTE_CONFIG_TS_EXTENSIONS = ['ts', 'mts'];
const cache = new Map();
/**
 * This function encapsulates the import call in a way
 * that TypeScript does not transpile `import()`.
 * https://github.com/microsoft/TypeScript/issues/43329
 */
const dynamicImport = new Function('modulePath', 'return import(modulePath)');
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
function loadConfig(dirOrFile, { traverse = true, clearCache = false } = {}) {
    if (clearCache)
        cache.clear();
    const resolved = node_path_1.default.resolve(dirOrFile);
    const cached = cache.get(resolved);
    if (cached) {
        return cached;
    }
    const loading = isFile(resolved)
        ? loadConfigFromFile(resolved)
        : loadConfigUncached(resolved, traverse);
    cache.set(resolved, loading);
    return loading;
}
function isFile(filePath) {
    try {
        return node_fs_1.default.statSync(filePath).isFile();
    }
    catch {
        return false;
    }
}
async function loadConfigFromFile(configFilePath) {
    const basename = node_path_1.default.basename(configFilePath);
    const root = node_path_1.default.dirname(configFilePath);
    if (/^svelte\.config\./.test(basename)) {
        return (await loadSvelteConfig(configFilePath)) ?? undefined;
    }
    const viteResult = await loadSvelteConfigFromVite(root, configFilePath);
    if (viteResult !== undefined) {
        return viteResult;
    }
    return loadSvelteConfig(configFilePath);
}
async function loadConfigUncached(dir, traverse) {
    let currentDir = dir;
    const dirs = [dir];
    while (true) {
        const result = await loadConfigFromDirectory(currentDir);
        if (result) {
            if (isLoadedConfig(result)) {
                // Cache the loaded config for all traversed directories
                for (const d of dirs) {
                    cache.set(d, Promise.resolve(result));
                }
            }
            else {
                cache.delete(dir);
            }
            return result;
        }
        if (!traverse) {
            return undefined;
        }
        const parentDir = node_path_1.default.dirname(currentDir);
        if (parentDir === currentDir) {
            return undefined;
        }
        currentDir = parentDir;
        dirs.push(currentDir);
    }
}
async function loadConfigFromDirectory(dir) {
    const viteConfigPath = findConfigInDirectory(dir, 'vite.config', VITE_CONFIG_EXTENSIONS);
    let viteError;
    if (viteConfigPath) {
        const result = await loadSvelteConfigFromVite(dir, viteConfigPath);
        if (isLoadedConfig(result)) {
            return result;
        }
        if (result?.error) {
            viteError = result;
        }
    }
    const svelteConfigPath = findConfigInDirectory(dir, 'svelte.config', getSvelteConfigExtensions());
    if (!svelteConfigPath) {
        return viteError;
    }
    return (await loadSvelteConfig(svelteConfigPath)) ?? viteError;
}
let resolving = null;
async function loadSvelteConfigFromVite(root, configFilePath) {
    const vite = await tryImportVite(root);
    if (!vite) {
        return undefined;
    }
    // Make sure that only one Vite config is resolved at a time, to prevent race conditions with multiple
    // calls to `loadConfig` ending up with changing the process' current working directory mid-resolution.
    await resolving;
    let resolve;
    resolving = new Promise((r) => (resolve = r));
    const cwd = process.cwd();
    try {
        process.chdir(root);
        const resolved = await vite.resolveConfig({ root, configFile: configFilePath, logLevel: 'error' }, 'serve');
        const kitPlugin = resolved.plugins.find((plugin) => plugin.name === 'vite-plugin-sveltekit-setup');
        const kitOptions = kitPlugin?.api?.options;
        if (kitOptions) {
            const { preprocess, compilerOptions, extensions, vitePlugin, ...kit } = kitOptions;
            return {
                config: { preprocess, compilerOptions, extensions, vitePlugin, kit },
                configFilePath,
                configSource: 'vite'
            };
        }
        const sveltePlugin = resolved.plugins.find((plugin) => plugin.name === 'vite-plugin-svelte:config');
        const options = sveltePlugin?.api?.options;
        if (options) {
            return {
                config: options,
                configFilePath,
                configSource: 'vite'
            };
        }
    }
    catch (error) {
        return {
            error,
            configFilePath,
            configSource: 'vite'
        };
    }
    finally {
        process.chdir(cwd);
        resolve();
    }
}
async function loadSvelteConfig(configFilePath) {
    try {
        const config = (await dynamicImport((0, node_url_1.pathToFileURL)(configFilePath).href))?.default;
        if (!config) {
            throw new Error('Missing exports in the config. Make sure to include "export default config" or "module.exports = config"');
        }
        return {
            config,
            configFilePath,
            configSource: 'svelte'
        };
    }
    catch (error) {
        return {
            error,
            configFilePath,
            configSource: 'svelte'
        };
    }
}
async function tryImportVite(fromPath) {
    try {
        const importPath = getViteImportPath(fromPath);
        if (importPath) {
            return await dynamicImport((0, node_url_1.pathToFileURL)(importPath).href);
        }
    }
    catch {
        // fall through to legacy import
    }
    return importViteLegacy(fromPath);
}
function getViteImportPath(fromPath) {
    const pkgPath = require.resolve('vite/package.json', { paths: [fromPath] });
    const pkgDir = node_path_1.default.dirname(pkgPath);
    const pkg = JSON.parse(node_fs_1.default.readFileSync(pkgPath, 'utf8'));
    const entry = resolvePackageImportExport(pkg.exports?.['.']);
    if (entry) {
        return node_path_1.default.join(pkgDir, entry);
    }
    const fallback = pkg.module ?? pkg.main;
    return fallback ? node_path_1.default.join(pkgDir, fallback) : undefined;
}
function resolvePackageImportExport(exportEntry) {
    if (typeof exportEntry === 'string') {
        return exportEntry;
    }
    if (!exportEntry || typeof exportEntry !== 'object') {
        return undefined;
    }
    const entry = exportEntry;
    const importEntry = entry.import;
    if (typeof importEntry === 'string') {
        return importEntry;
    }
    if (importEntry && typeof importEntry === 'object') {
        const defaultEntry = importEntry.default;
        if (typeof defaultEntry === 'string') {
            return defaultEntry;
        }
    }
    if (typeof entry.default === 'string') {
        return entry.default;
    }
    return undefined;
}
async function importViteLegacy(fromPath) {
    try {
        const main = require.resolve('vite', { paths: [fromPath] });
        // require.resolve will use the cjs version
        const prev = process.env.VITE_CJS_IGNORE_WARNING;
        process.env.VITE_CJS_IGNORE_WARNING = 'true';
        const result = await dynamicImport((0, node_url_1.pathToFileURL)(main).href);
        process.env.VITE_CJS_IGNORE_WARNING = prev;
        return result;
    }
    catch {
        return undefined;
    }
}
function findConfigInDirectory(dir, basename, extensions) {
    for (const extension of extensions) {
        const configPath = node_path_1.default.join(dir, `${basename}.${extension}`);
        if (node_fs_1.default.existsSync(configPath)) {
            return configPath;
        }
    }
}
function getSvelteConfigExtensions() {
    return process.features && 'typescript' in process.features && process.features.typescript
        ? [...SVELTE_CONFIG_EXTENSIONS, ...SVELTE_CONFIG_TS_EXTENSIONS]
        : SVELTE_CONFIG_EXTENSIONS;
}
function isLoadedConfig(result) {
    return !!result && 'config' in result;
}
//# sourceMappingURL=index.js.map