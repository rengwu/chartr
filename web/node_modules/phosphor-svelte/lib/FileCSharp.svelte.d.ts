import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileCSharpIcon` instead.
 *
 * @example
 * ```svelte
 * <FileCSharp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileCSharp: Component<IconComponentProps, {}, "">;
type FileCSharp = ReturnType<typeof FileCSharp>;
export default FileCSharp;
