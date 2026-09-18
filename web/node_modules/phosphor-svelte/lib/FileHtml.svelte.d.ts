import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileHtmlIcon` instead.
 *
 * @example
 * ```svelte
 * <FileHtml color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileHtml: Component<IconComponentProps, {}, "">;
type FileHtml = ReturnType<typeof FileHtml>;
export default FileHtml;
