import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TableIcon` instead.
 *
 * @example
 * ```svelte
 * <Table color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Table: Component<IconComponentProps, {}, "">;
type Table = ReturnType<typeof Table>;
export default Table;
