import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DownloadSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <DownloadSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DownloadSimple: Component<IconComponentProps, {}, "">;
type DownloadSimple = ReturnType<typeof DownloadSimple>;
export default DownloadSimple;
