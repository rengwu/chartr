import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileTxtIcon` instead.
 *
 * @example
 * ```svelte
 * <FileTxt color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileTxt: Component<IconComponentProps, {}, "">;
type FileTxt = ReturnType<typeof FileTxt>;
export default FileTxt;
