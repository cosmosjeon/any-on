import { FieldTemplateProps } from '@rjsf/utils';

export const FieldTemplate = (props: FieldTemplateProps) => {
  const {
    children,
    rawErrors = [],
    rawHelp,
    rawDescription,
    label,
    required,
    schema,
  } = props;

  if (schema.type === 'object') {
    return children;
  }

  // Single-column layout for boolean/checkbox fields
  if (schema.type === 'boolean' || (Array.isArray(schema.type) && schema.type.includes('boolean'))) {
    return (
      <div className="py-3 space-y-2">
        <div className="flex items-start gap-3">
          <div className="pt-0.5">{children}</div>
          <div className="flex-1 space-y-1">
            {label && (
              <label 
                htmlFor={props.id} 
                className="text-sm font-medium leading-none cursor-pointer hover:text-foreground transition-colors"
              >
                {label}
                {required && <span className="text-destructive ml-1">*</span>}
              </label>
            )}

            {rawDescription && (
              <p className="text-sm text-muted-foreground leading-relaxed">
                {rawDescription}
              </p>
            )}

            {rawHelp && (
              <p className="text-sm text-muted-foreground leading-relaxed">
                {rawHelp}
              </p>
            )}

            {rawErrors.length > 0 && (
              <div className="space-y-1 pt-1">
                {rawErrors.map((error, index) => (
                  <p key={index} className="text-sm text-destructive">
                    {error}
                  </p>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Two-column layout for other field types
  return (
    <div className="grid grid-cols-1 md:grid-cols-[200px_1fr] gap-6 py-4">
      {/* Left column: Label and description */}
      <div className="space-y-1.5">
        {label && (
          <label 
            htmlFor={props.id}
            className="text-sm font-semibold leading-tight block"
          >
            {label}
            {required && <span className="text-destructive ml-1">*</span>}
          </label>
        )}

        {rawDescription && (
          <p className="text-xs text-muted-foreground leading-relaxed">
            {rawDescription}
          </p>
        )}

        {rawHelp && (
          <p className="text-xs text-muted-foreground leading-relaxed">
            {rawHelp}
          </p>
        )}
      </div>

      {/* Right column: Field content */}
      <div className="space-y-2">
        {children}

        {rawErrors.length > 0 && (
          <div className="space-y-1 pt-1">
            {rawErrors.map((error, index) => (
              <p key={index} className="text-sm text-destructive">
                {error}
              </p>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
