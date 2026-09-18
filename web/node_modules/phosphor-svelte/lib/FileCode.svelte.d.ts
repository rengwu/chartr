import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileCodeIcon` instead.
 *
 * @example
 * ```svelte
 * <FileCode color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileCode: Component<IconComponentProps, {}, "">;
type FileCode = ReturnType<typeof FileCode>;
export default FileCode;
