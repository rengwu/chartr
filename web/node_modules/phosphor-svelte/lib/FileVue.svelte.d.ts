import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileVueIcon` instead.
 *
 * @example
 * ```svelte
 * <FileVue color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileVue: Component<IconComponentProps, {}, "">;
type FileVue = ReturnType<typeof FileVue>;
export default FileVue;
