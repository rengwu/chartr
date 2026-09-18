import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CampfireIcon` instead.
 *
 * @example
 * ```svelte
 * <Campfire color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Campfire: Component<IconComponentProps, {}, "">;
type Campfire = ReturnType<typeof Campfire>;
export default Campfire;
