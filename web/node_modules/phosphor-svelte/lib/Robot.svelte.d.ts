import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RobotIcon` instead.
 *
 * @example
 * ```svelte
 * <Robot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Robot: Component<IconComponentProps, {}, "">;
type Robot = ReturnType<typeof Robot>;
export default Robot;
