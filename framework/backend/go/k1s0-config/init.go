package k1s0config

// ServiceInit provides a convenient way to initialize a service configuration.
type ServiceInit struct {
	args   *ServiceArgs
	loader *ConfigLoader
}

// NewServiceInit creates a new ServiceInit from ServiceArgs.
func NewServiceInit(args *ServiceArgs) (*ServiceInit, error) {
	if err := args.Validate(); err != nil {
		return nil, err
	}

	opts := args.ToConfigOptions()
	loader, err := NewConfigLoader(opts)
	if err != nil {
		return nil, err
	}

	return &ServiceInit{
		args:   args,
		loader: loader,
	}, nil
}

// Load loads the configuration into the target.
func (s *ServiceInit) Load(target interface{}) error {
	return s.loader.Load(target)
}

// Loader returns the underlying ConfigLoader.
func (s *ServiceInit) Loader() *ConfigLoader {
	return s.loader
}

// Env returns the environment name.
func (s *ServiceInit) Env() string {
	return s.args.Env
}

// ResolveSecret resolves a secret from a file reference.
func (s *ServiceInit) ResolveSecret(fileValue, key string) (string, error) {
	return s.loader.ResolveSecretFile(fileValue, key)
}

// Init is a convenience function to initialize and load configuration in one step.
func Init(args *ServiceArgs, target interface{}) (*ServiceInit, error) {
	init, err := NewServiceInit(args)
	if err != nil {
		return nil, err
	}

	if err := init.Load(target); err != nil {
		return nil, err
	}

	return init, nil
}
