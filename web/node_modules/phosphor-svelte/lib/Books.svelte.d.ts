import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BooksIcon` instead.
 *
 * @example
 * ```svelte
 * <Books color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Books: Component<IconComponentProps, {}, "">;
type Books = ReturnType<typeof Books>;
export default Books;
