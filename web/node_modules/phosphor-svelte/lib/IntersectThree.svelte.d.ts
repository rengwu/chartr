import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IntersectThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <IntersectThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const IntersectThree: Component<IconComponentProps, {}, "">;
type IntersectThree = ReturnType<typeof IntersectThree>;
export default IntersectThree;
