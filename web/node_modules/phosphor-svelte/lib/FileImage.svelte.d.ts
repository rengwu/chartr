import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileImageIcon` instead.
 *
 * @example
 * ```svelte
 * <FileImage color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileImage: Component<IconComponentProps, {}, "">;
type FileImage = ReturnType<typeof FileImage>;
export default FileImage;
