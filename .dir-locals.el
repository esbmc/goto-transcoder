((nil . ((eval . (progn
                   (let ((project-root (locate-dominating-file default-directory ".dir-locals.el")))
                     (when project-root
                       (load (expand-file-name "goto-transcoder-mode.el" project-root) nil t)
                       (goto-transcoder-mode 1))))))))
