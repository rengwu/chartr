import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileSqlIcon` instead.
 *
 * @example
 * ```svelte
 * <FileSql color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileSql: Component<IconComponentProps, {}, "">;
type FileSql = ReturnType<typeof FileSql>;
export default FileSql;
