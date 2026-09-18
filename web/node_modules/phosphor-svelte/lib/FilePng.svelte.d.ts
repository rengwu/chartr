import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilePngIcon` instead.
 *
 * @example
 * ```svelte
 * <FilePng color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilePng: Component<IconComponentProps, {}, "">;
type FilePng = ReturnType<typeof FilePng>;
export default FilePng;
