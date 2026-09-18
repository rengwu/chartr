import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeadlightsIcon` instead.
 *
 * @example
 * ```svelte
 * <Headlights color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Headlights: Component<IconComponentProps, {}, "">;
type Headlights = ReturnType<typeof Headlights>;
export default Headlights;
