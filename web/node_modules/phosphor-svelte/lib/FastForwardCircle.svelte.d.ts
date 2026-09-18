import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FastForwardCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <FastForwardCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FastForwardCircle: Component<IconComponentProps, {}, "">;
type FastForwardCircle = ReturnType<typeof FastForwardCircle>;
export default FastForwardCircle;
