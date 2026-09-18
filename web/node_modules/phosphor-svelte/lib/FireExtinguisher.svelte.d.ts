import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FireExtinguisherIcon` instead.
 *
 * @example
 * ```svelte
 * <FireExtinguisher color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FireExtinguisher: Component<IconComponentProps, {}, "">;
type FireExtinguisher = ReturnType<typeof FireExtinguisher>;
export default FireExtinguisher;
