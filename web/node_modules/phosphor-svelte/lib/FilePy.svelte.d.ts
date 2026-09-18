import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilePyIcon` instead.
 *
 * @example
 * ```svelte
 * <FilePy color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilePy: Component<IconComponentProps, {}, "">;
type FilePy = ReturnType<typeof FilePy>;
export default FilePy;
