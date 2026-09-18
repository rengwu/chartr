import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MeteorIcon` instead.
 *
 * @example
 * ```svelte
 * <Meteor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Meteor: Component<IconComponentProps, {}, "">;
type Meteor = ReturnType<typeof Meteor>;
export default Meteor;
