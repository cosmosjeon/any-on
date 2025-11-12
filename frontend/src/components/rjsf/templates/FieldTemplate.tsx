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
  if (schema.type === 'boolean') {
    return (
      <div className="py-4 space-y-2">
        {children}

        {rawDescription && (
          <p className="text-sm text-muted-foreground leading-relaxed ml-6">
            {rawDescription}
          </p>
        )}

        {rawHelp && (
          <p className="text-sm text-muted-foreground leading-relaxed ml-6">
            {rawHelp}
          </p>
        )}

        {rawErrors.length > 0 && (
          <div className="space-y-1 ml-6">
            {rawErrors.map((error, index) => (
              <p key={index} className="text-sm text-destructive">
                {error}
              </p>
            ))}
          </div>
        )}
      </div>
    );
  }

  // Two-column layout for other field types
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 py-6">
      {/* Left column: Label and description */}
      <div className="space-y-2">
        {label && (
          <div className="text-sm font-bold leading-relaxed">
            {label}
            {required && <span className="text-destructive ml-1">*</span>}
          </div>
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
      </div>

      {/* Right column: Field content */}
      <div className="space-y-2">
        {children}

        {rawErrors.length > 0 && (
          <div className="space-y-1">
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
