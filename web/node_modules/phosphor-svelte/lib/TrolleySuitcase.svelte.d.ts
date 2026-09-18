import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrolleySuitcaseIcon` instead.
 *
 * @example
 * ```svelte
 * <TrolleySuitcase color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TrolleySuitcase: Component<IconComponentProps, {}, "">;
type TrolleySuitcase = ReturnType<typeof TrolleySuitcase>;
export default TrolleySuitcase;
