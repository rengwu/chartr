import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrashSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <TrashSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrashSimple: Component<IconComponentProps, {}, "">;
type TrashSimple = ReturnType<typeof TrashSimple>;
export default TrashSimple;
