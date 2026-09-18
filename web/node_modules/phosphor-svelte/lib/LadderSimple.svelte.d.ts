import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LadderSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <LadderSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LadderSimple: Component<IconComponentProps, {}, "">;
type LadderSimple = ReturnType<typeof LadderSimple>;
export default LadderSimple;
