import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DatabaseIcon` instead.
 *
 * @example
 * ```svelte
 * <Database color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Database: Component<IconComponentProps, {}, "">;
type Database = ReturnType<typeof Database>;
export default Database;
