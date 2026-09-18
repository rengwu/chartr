import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StandardDefinitionIcon` instead.
 *
 * @example
 * ```svelte
 * <StandardDefinition color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StandardDefinition: Component<IconComponentProps, {}, "">;
type StandardDefinition = ReturnType<typeof StandardDefinition>;
export default StandardDefinition;
