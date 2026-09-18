import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilePdfIcon` instead.
 *
 * @example
 * ```svelte
 * <FilePdf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilePdf: Component<IconComponentProps, {}, "">;
type FilePdf = ReturnType<typeof FilePdf>;
export default FilePdf;
