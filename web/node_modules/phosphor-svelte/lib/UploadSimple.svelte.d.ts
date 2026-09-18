import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UploadSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <UploadSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const UploadSimple: Component<IconComponentProps, {}, "">;
type UploadSimple = ReturnType<typeof UploadSimple>;
export default UploadSimple;
