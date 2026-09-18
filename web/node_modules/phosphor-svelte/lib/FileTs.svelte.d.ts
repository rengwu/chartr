import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileTsIcon` instead.
 *
 * @example
 * ```svelte
 * <FileTs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileTs: Component<IconComponentProps, {}, "">;
type FileTs = ReturnType<typeof FileTs>;
export default FileTs;
