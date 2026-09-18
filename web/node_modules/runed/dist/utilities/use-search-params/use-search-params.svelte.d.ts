/**
 * Configuration options for useSearchParams
 */
export interface SearchParamsOptions {
    /**
     * If true, parameters set to their default values will be shown in the URL.
     * If false, parameters with default values will be omitted from the URL.
     * @default false
     */
    showDefaults?: boolean;
    /**
     * The number of milliseconds to delay URL updates when parameters change.
     * This helps avoid cluttering browser history when values change rapidly
     * (like during typing in an input field).
     * @default 0 (no debounce)
     */
    debounce?: number;
    /**
     * Controls whether URL updates create new browser history entries.
     * If true (default), each update adds a new entry to the browser history.
     * If false, updates replace the current URL without creating new history entries.
     * @default true
     */
    pushHistory?: boolean;
    /**
     * Enable lz-string compression for all parameters.
     * When true, all parameters are compressed into a single parameter in the URL.
     * This helps reduce URL length and provides basic parameter obfuscation.
     * @default false
     */
    compress?: boolean;
    /**
     * The name of the parameter used to store compressed data when compression is enabled.
     * You can customize this to avoid conflicts with your schema parameters.
     *
     * For example, if your schema already uses '_data', you might want to use '_compressed'
     * or another unique name.
     *
     * @default '_data'
     */
    compressedParamName?: string;
    /**
     * Controls whether to update the URL when parameters change.
     * If `true` (default), changes to parameters will update the URL.
     * If `false`, parameters will only be stored in memory without updating the URL.
     *
     * Note: When `false`, compress option will be ignored.
     * @default true
     */
    updateURL?: boolean;
    /**
     * If `true`, the scroll position will be preserved when the URL is updated.
     *
     * If `false`, the scroll position will be reset to the top when the URL is updated.
     * @default false
     */
    noScroll?: boolean;
}
/** The Standard Schema interface. */
export interface StandardSchemaV1<Input = unknown, Output = Input> {
    /** The Standard Schema properties. */
    readonly "~standard": StandardSchemaV1.Props<Input, Output>;
}
export declare namespace StandardSchemaV1 {
    /** The Standard Schema properties interface. */
    interface Props<Input = unknown, Output = Input> {
        /** The version number of the standard. */
        readonly version: 1;
        /** The vendor name of the schema library. */
        readonly vendor: string;
        /** Validates unknown input values. */
        readonly validate: (value: unknown) => Result<Output> | Promise<Result<Output>>;
        /** Inferred types associated with the schema. */
        readonly types?: Types<Input, Output> | undefined;
    }
    /** The result interface of the validate function. */
    type Result<T> = SuccessResult<T> | FailureResult;
    /** The result interface if validation succeeds. */
    interface SuccessResult<T> {
        /** The typed output value. */
        readonly value: T;
        /** The non-existent issues. */
        readonly issues?: undefined;
    }
    /** The result interface if validation fails. */
    interface FailureResult {
        /** The issues of failed validation. */
        readonly issues: ReadonlyArray<Issue>;
    }
    /** The issue interface of the failure output. */
    interface Issue {
        /** The error message of the issue. */
        readonly message: string;
        /** The path of the issue, if any. */
        readonly path?: ReadonlyArray<PropertyKey | PathSegment> | undefined;
    }
    /** The path segment interface of the issue. */
    interface PathSegment {
        /** The key representing a path segment. */
        readonly key: PropertyKey;
    }
    /** The Standard Schema types interface. */
    interface Types<Input = unknown, Output = Input> {
        /** The input type of the schema. */
        readonly input: Input;
        /** The output type of the schema. */
        readonly output: Output;
    }
    /** Infers the input type of a Standard Schema. */
    type InferInput<Schema extends StandardSchemaV1> = NonNullable<Schema["~standard"]["types"]>["input"];
    /** Infers the output type of a Standard Schema. */
    type InferOutput<Schema extends StandardSchemaV1> = NonNullable<Schema["~standard"]["types"]>["output"];
}
/**
 * Core class that handles URL search parameter operations with schema validation
 *
 * This class provides the foundation for the useSearchParams hook. It:
 * 1. Validates values against a schema
 * 2. Handles type conversion between URL strings and JavaScript types
 * 3. Updates the URL when values change
 * 4. Maintains a cache of valid schema keys for performance
 */
declare class SearchParams<Schema extends StandardSchemaV1> {
    #private;
    /**
     * Create a new SearchParams instance with the given schema and options
     *
     * @param schema A StandardSchemaV1-compatible schema
     * @param options Configuration options
     */
    constructor(schema: Schema, options?: SearchParamsOptions);
    /**
     * Get a typed parameter value by key
     * Retrieves the current value from the local cache, runs it through schema validation,
     * and returns the validated, typed result
     *
     * @param key The parameter key to get
     * @returns The typed value after schema validation
     */
    get<K extends keyof StandardSchemaV1.InferOutput<Schema>>(key: K & string): StandardSchemaV1.InferOutput<Schema>[K];
    /**
     * Set a parameter value and update the URL
     * Validates the value through the schema before updating the URL
     *
     * @param key The parameter key to set
     * @param value The value to set (will be type-converted and validated)
     */
    set<K extends keyof StandardSchemaV1.InferOutput<Schema>>(key: K & string, value: StandardSchemaV1.InferOutput<Schema>[K]): void;
    /**
     * Clean up resources used by this instance
     *
     * IMPORTANT: You only need to call this method when using the debounce option.
     * If you're not using debounce, there's no need to call cleanup.
     *
     * Call this when the component unmounts to prevent memory leaks from debounce timers.
     *
     * @example
     * Example in a Svelte component with Svelte 5 runes:
     * ```svelte
     * <script>
     *   import { useSearchParams } from '../../hooks/useSearchParams.svelte';
     *
     *   // Using debounce, so we need to handle cleanup
     *   const searchParams = useSearchParams(schema, { debounce: 300 });
     *
     *   // Register cleanup in a Svelte 5 effect
     *   $effect(() => {
     *     return () => {
     *       // Prevent memory leaks by cleaning up debounce timer
     *       searchParams.cleanup();
     *     };
     *   });
     * </script>
     * ```
     */
    cleanup(): void;
    /**
     * Update multiple parameters at once
     *
     * This is more efficient than setting multiple parameters individually
     * because it only triggers one URL update or one in-memory store update.
     *
     * @param values An object containing parameter key-value pairs to update
     */
    update(values: Partial<StandardSchemaV1.InferOutput<Schema>>): void;
    /**
     * Check if a key exists in the schema
     * This is a critical method used by the Proxy handler to determine
     * which properties should be treated as URL parameters
     *
     * @param key The key to check
     * @returns True if the key is defined in the schema
     */
    has(key: string): boolean;
    /**
     * Reset all parameters to their default values
     *
     * This method removes all current URL parameters or in-memory parameters
     * and optionally sets parameters with non-default values back to their defaults.
     *
     * @param showDefaults Whether to show default values in the URL or in-memory store after reset.
     *                     If not provided, uses the instance's showDefaults option.
     */
    reset(showDefaults?: boolean): void;
    /**
     * Validate a value against the schema
     * This is the core method that enforces schema validation
     *
     * @param value The value to validate
     * @returns A StandardSchemaV1.Result containing either the validated value or validation errors
     */
    validate(value: unknown): StandardSchemaV1.Result<StandardSchemaV1.InferOutput<Schema>>;
}
/**
 * Schema type for createSearchParamsSchema
 * Allows specifying more precise types for arrays and objects
 */
export type SchemaTypeConfig<ArrayType = unknown, ObjectType = unknown> = {
    type: "string";
    default?: string;
} | {
    type: "number";
    default?: number;
} | {
    type: "boolean";
    default?: boolean;
} | {
    type: "array";
    default?: ArrayType[];
    arrayType?: ArrayType;
} | {
    type: "object";
    default?: ObjectType;
    objectType?: ObjectType;
};
/**
 * Creates a simple schema compatible with useSearchParams without requiring external validation libraries.
 *
 * This is a lightweight alternative to using full schema validation libraries like Zod, Valibot, or Arktype.
 * Use this when you need basic type conversion and default values without adding dependencies.
 *
 * Limitations:
 * - For 'array' type: supports basic arrays, but doesn't validate array items
 * - For 'object' type: supports generic objects, but doesn't validate nested properties
 * - No custom validation rules or transformations
 * - No granular reactivity: nested property changes require whole-value reassignment
 *   (e.g., params.items = [...params.items, newItem] instead of params.items.push(newItem))
 *
 * For complex validation needs (nested validation, refined rules, etc.), use a dedicated
 * validation library instead.
 *
 * Example usage:
 * ```
 * const productSearchSchema = createSearchParamsSchema({
 *   // Basic types with defaults
 *   page: { type: 'number', default: 1 },
 *   filter: { type: 'string', default: '' },
 *   sort: { type: 'string', default: 'newest' },
 *
 *   // Array type with specific element type
 *   tags: {
 *     type: 'array',
 *     default: ['new'],
 *     arrayType: '' // Specify string[] type
 *   },
 *
 *   // Object type with specific shape
 *   config: {
 *     type: 'object',
 *     default: { theme: 'light' },
 *     objectType: { theme: '' } // Specify { theme: string } type
 *   }
 * });
 * ```
 *
 * URL storage format:
 * - Arrays are stored as JSON strings: ?tags=["sale","featured"]
 * - Objects are stored as JSON strings: ?config={"theme":"dark","fontSize":14}
 * - Primitive values are stored directly: ?page=2&filter=red
 */
export declare function createSearchParamsSchema<T extends Record<string, SchemaTypeConfig>>(schema: T): StandardSchemaV1<unknown, {
    [K in keyof T]: T[K] extends {
        type: "number";
    } ? number : T[K] extends {
        type: "boolean";
    } ? boolean : T[K] extends {
        type: "array";
        arrayType?: infer A;
    } ? unknown extends A ? unknown[] : A[] : T[K] extends {
        type: "object";
        objectType?: infer O;
    } ? unknown extends O ? Record<string, unknown> : O : string;
}>;
/**
 * A utility function to extract, validate and convert URL search parameters to [URLSearchParams](https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams)
 *
 * This function makes it easy to use the same schema validation in
 * both client-side components (via useSearchParams) and server-side load functions.
 * Unlike useSearchParams, this function doesn't modify the URL - it only validates
 * parameters and returns them as a new URLSearchParams object.
 *
 * **Important for SvelteKit fine-grained reactivity**: This function only accesses URL parameters
 * that are defined in your schema, ensuring that load functions only re-run when schema-defined
 * parameters change, not when any URL parameter changes.
 *
 * Handles both standard URL parameters and compressed parameters (when compression is enabled).
 *
 * @param url The URL object from SvelteKit load function
 * @param schema A validation schema (createSearchParamsSchema, Zod, Valibot, etc.)
 * @param options Optional configuration (like custom compressedParamName)
 * @returns URLSearchParams object with the validated values
 *
 * Example with SvelteKit page or layout load function:
 * ```ts
 * import { validateSearchParams } from '../../hooks/useSearchParams.svelte';
 * import { productSchema } from './schemas';
 *
 * export const load = ({ url }) => {
 *   // Get validated search params as URLSearchParams object
 *   // Only accesses 'page', 'filter', 'sort' parameters from the URL
 *   // Load function will only re-run when these specific parameters change
 *   const searchParams = validateSearchParams(url, productSchema, {
 *     compressedParamName: '_compressed'
 *   });
 *
 *   // Use URLSearchParams directly with fetch
 *   const response = await fetch(`/api/products?${searchParams.toString()}`);
 *   return {
 *     products: await response.json()
 *   };
 * };
 * ```
 */
export declare function validateSearchParams<Schema extends StandardSchemaV1>(url: URL, schema: Schema, options?: {
    compressedParamName?: string;
}): {
    searchParams: URLSearchParams;
    data: StandardSchemaV1.InferOutput<Schema>;
};
export type ReturnUseSearchParams<T extends StandardSchemaV1> = SearchParams<T> & StandardSchemaV1.InferOutput<T> & {
    /**
     * Convert the current schema parameters to a URLSearchParams object
     * This includes all values defined in the schema, regardless of their presence in the URL
     * @returns URLSearchParams object containing all current parameter values
     */
    toURLSearchParams(): URLSearchParams;
};
/**
 * Hook to create a reactive search params object with property access
 *
 * This client-side hook automatically updates the URL when parameters change.
 * It provides type-safe access to URL search parameters through direct property access.
 *
 * @param schema A validation schema compatible with StandardSchemaV1
 * @param options Configuration options that affect URL behavior
 * @returns A reactive object for working with typed search parameters
 *
 * Available options:
 * - `showDefaults` (boolean): When true, parameters with default values will be shown in the URL.
 *   When false (default), parameters with default values will be omitted from the URL.
 * - `debounce` (number): Milliseconds to delay URL updates when parameters change.
 *   Useful to avoid cluttering browser history when values change rapidly (default: 0, no debounce).
 * - `pushHistory` (boolean): Controls whether URL updates create new browser history entries.
 *   If true (default), each update adds a new entry to the browser history.
 *   If false, updates replace the current URL without creating new history entries.
 * - `compress` (boolean): When true, all parameters are compressed into a single parameter
 *   using lz-string compression. This helps reduce URL length and provides basic obfuscation (default: false).
 *   Use validateSearchParams with the same compressedParamName option when handling compressed URLs server-side.
 * - `compressedParamName` (string): The name of the parameter used to store compressed data
 *   when compression is enabled. Customize this to avoid conflicts with parameters in your schema.
 *   Default is '_data'.
 * - `updateURL` (boolean): When true (default), the URL is updated when parameters change.
 *   When false, only in-memory parameters are updated.
 *
 * Example with Zod:
 * ```
 * import { z } from 'zod';
 *
 * const productSearchSchema = z.object({
 *   page: z.number().catch(1),
 *   filter: z.string().catch(''),
 *   sort: z.enum(['newest', 'oldest', 'price']).catch('newest'),
 * });
 *
 * const params = useSearchParams(productSearchSchema);
 *
 * // Access parameters directly
 * const page = $derived(params.page); // number (defaults to 1)
 * const sort = $derived(params.sort); // 'newest' | 'oldest' | 'price'
 *
 * // Update parameters directly
 * params.page = 2; // Updates URL to include ?page=2
 * params.sort = 'price'; // Updates URL to include &sort=price
 * ```
 *
 * Example with options:
 * ```typescript
 * // Show default values in URL, debounce updates by 300ms,
 * // don't create new history entries, and compress params
 * const params = useSearchParams(schema, {
 *   showDefaults: true,
 *   debounce: 300,
 *   pushHistory: false,
 *   compress: true,
 *   compressedParamName: '_compressed' // Custom name to avoid conflicts
 * });
 *
 * // Great for binding to input fields (updates URL without cluttering history)
 * <input type="text" bind:value={params.search} />
 * // Resulting URL will be something like: /?_compressed=N4IgDgTg9g...
 * ```
 * Example with Valibot:
 * ```
 * import * as v from 'valibot';
 *
 * const productSearchSchema = v.object({
 *   page: v.optional(v.fallback(v.number(), 1), 1),
 *   filter: v.optional(v.fallback(v.string(), ''), ''),
 *   sort: v.optional(v.fallback(v.picklist(['newest', 'oldest', 'price']), 'newest'), 'newest'),
 * });
 *
 * const params = useSearchParams(productSearchSchema);
 * ``` * Example with Arktype:
 * ```
 * import { type } from 'arktype';
 *
 * const productSearchSchema = type({
 *   page: 'number = 1',
 *   filter: 'string = ""',
 *   sort: '"newest" | "oldest" | "price" = "newest"',
 * });
 *
 * const params = useSearchParams(productSearchSchema);
 * ```
 * Or with our built-in schema creator (no additional dependencies):
 *
 * ```
 * const productSearchSchema = createSearchParamsSchema({
 *   page: { type: 'number', default: 1 },
 *   filter: { type: 'string', default: '' },
 *   sort: { type: 'string', default: 'newest' }
 * });
 *
 * const params = useSearchParams(productSearchSchema);
 * ```
 */
export declare function useSearchParams<Schema extends StandardSchemaV1>(schema: Schema, options?: SearchParamsOptions): ReturnUseSearchParams<Schema>;
export {};
