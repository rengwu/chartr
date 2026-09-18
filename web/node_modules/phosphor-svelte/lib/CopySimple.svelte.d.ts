import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CopySimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <CopySimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CopySimple: Component<IconComponentProps, {}, "">;
type CopySimple = ReturnType<typeof CopySimple>;
export default CopySimple;
