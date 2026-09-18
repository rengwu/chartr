import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileDocIcon` instead.
 *
 * @example
 * ```svelte
 * <FileDoc color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileDoc: Component<IconComponentProps, {}, "">;
type FileDoc = ReturnType<typeof FileDoc>;
export default FileDoc;
