import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileMdIcon` instead.
 *
 * @example
 * ```svelte
 * <FileMd color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileMd: Component<IconComponentProps, {}, "">;
type FileMd = ReturnType<typeof FileMd>;
export default FileMd;
