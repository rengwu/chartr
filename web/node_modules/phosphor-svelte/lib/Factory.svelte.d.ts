import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FactoryIcon` instead.
 *
 * @example
 * ```svelte
 * <Factory color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Factory: Component<IconComponentProps, {}, "">;
type Factory = ReturnType<typeof Factory>;
export default Factory;
