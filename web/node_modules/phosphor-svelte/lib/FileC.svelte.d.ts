import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileCIcon` instead.
 *
 * @example
 * ```svelte
 * <FileC color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileC: Component<IconComponentProps, {}, "">;
type FileC = ReturnType<typeof FileC>;
export default FileC;
