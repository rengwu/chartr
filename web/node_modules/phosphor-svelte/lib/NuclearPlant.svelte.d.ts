import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NuclearPlantIcon` instead.
 *
 * @example
 * ```svelte
 * <NuclearPlant color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NuclearPlant: Component<IconComponentProps, {}, "">;
type NuclearPlant = ReturnType<typeof NuclearPlant>;
export default NuclearPlant;
