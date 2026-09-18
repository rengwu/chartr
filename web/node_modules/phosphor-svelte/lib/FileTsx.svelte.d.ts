import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileTsxIcon` instead.
 *
 * @example
 * ```svelte
 * <FileTsx color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileTsx: Component<IconComponentProps, {}, "">;
type FileTsx = ReturnType<typeof FileTsx>;
export default FileTsx;
