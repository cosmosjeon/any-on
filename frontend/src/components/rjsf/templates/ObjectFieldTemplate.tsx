import { ObjectFieldTemplateProps } from '@rjsf/utils';

export const ObjectFieldTemplate = (props: ObjectFieldTemplateProps) => {
  const { properties } = props;

  return (
    <div className="space-y-1">
      {properties.map((element) => (
        <div key={element.name} className="border-b border-border/50 last:border-0">
          {element.content}
        </div>
      ))}
    </div>
  );
};
