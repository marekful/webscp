package cmd

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/marekful/webscp/version"
)

func init() {
	rootCmd.AddCommand(versionCmd)
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print the version number",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("WebSCP v" + version.Version + "/" + version.CommitSHA)
	},
}
